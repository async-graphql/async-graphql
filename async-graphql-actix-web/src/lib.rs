//! Async-graphql integration with Actix-web

#![warn(missing_docs)]

mod subscription;

use actix_web::dev::{Payload, PayloadStream};
use actix_web::{http, web, Error, FromRequest, HttpRequest};
use async_graphql::http::StreamBody;
use async_graphql::{IntoQueryBuilder, IntoQueryBuilderOpts, QueryBuilder};
use futures::channel::mpsc;
use futures::{Future, SinkExt, StreamExt, TryFutureExt};
use std::pin::Pin;
pub use subscription::WSSubscription;

/// Extractor for GraphQL request
///
/// It's a wrapper of `QueryBuilder`, you can use `GQLRequest::into_inner` unwrap it to `QueryBuilder`.
/// `async_graphql::IntoQueryBuilderOpts` allows to configure extraction process.
pub struct GQLRequest(QueryBuilder);

impl GQLRequest {
    /// Unwrap it to `QueryBuilder`.
    pub fn into_inner(self) -> QueryBuilder {
        self.0
    }
}

impl FromRequest for GQLRequest {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<GQLRequest, Error>>>>;
    type Config = IntoQueryBuilderOpts;

    fn from_request(req: &HttpRequest, payload: &mut Payload<PayloadStream>) -> Self::Future {
        let config = req.app_data::<Self::Config>().cloned().unwrap_or_default();
        let content_type = req
            .headers()
            .get(http::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_string());

        let (mut tx, rx) = mpsc::channel(16);

        // Because Payload is !Send, so forward it to mpsc::Sender
        let mut payload = web::Payload(payload.take());
        actix_rt::spawn(async move {
            while let Some(item) = payload.next().await {
                if tx.send(item).await.is_err() {
                    return;
                }
            }
        });

        Box::pin(async move {
            (content_type, StreamBody::new(rx))
                .into_query_builder_opts(&config)
                .map_ok(GQLRequest)
                .map_err(actix_web::error::ErrorBadRequest)
                .await
        })
    }
}
