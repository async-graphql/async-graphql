//! Async-graphql integration with Actix-web

#![warn(missing_docs)]
#![forbid(unsafe_code)]

use std::pin::Pin;

use actix_web::dev::{HttpResponseBuilder, Payload, PayloadStream};
use actix_web::http::StatusCode;
use actix_web::{http, web, Error, FromRequest, HttpRequest, HttpResponse, Responder};
use futures::channel::mpsc;
use futures::future::Ready;
use futures::{Future, SinkExt, StreamExt, TryFutureExt};
use http::Method;

use async_graphql::http::StreamBody;
use async_graphql::{
    QueryDefinition, BatchQueryResponse, IntoQueryDefinition, IntoQueryBuilderOpts,
    ParseRequestError,
};
pub use subscription::WSSubscription;

mod subscription;

/// Extractor for GraphQL request
///
/// It's a wrapper of `BatchQueryBuilder`, you can use `BatchGQLRequest::into_inner` unwrap it to `BatchQueryBuilder`.
/// `async_graphql::IntoQueryBuilderOpts` allows to configure extraction process.
pub struct BatchGQLRequest(QueryDefinition);

impl BatchGQLRequest {
    /// Unwrap it to `QueryBuilder`.
    pub fn into_inner(self) -> QueryDefinition {
        self.0
    }
}

impl FromRequest for BatchGQLRequest {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<BatchGQLRequest, Error>>>>;
    type Config = IntoQueryBuilderOpts;

    fn from_request(req: &HttpRequest, payload: &mut Payload<PayloadStream>) -> Self::Future {
        let config = req.app_data::<Self::Config>().cloned().unwrap_or_default();

        if req.method() == Method::GET {
            let res =
                web::Query::<async_graphql::http::GQLRequest>::from_query(req.query_string());
            Box::pin(async move {
                let gql_request = res?;
                gql_request
                    .into_inner()
                    .into_batch_query_definition_opts(&config)
                    .map_ok(BatchGQLRequest)
                    .map_err(actix_web::error::ErrorBadRequest)
                    .await
            })
        } else {
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
                    .into_batch_query_definition_opts(&config)
                    .map_ok(BatchGQLRequest)
                    .map_err(|err| match err {
                        ParseRequestError::PayloadTooLarge => {
                            actix_web::error::ErrorPayloadTooLarge(err)
                        }
                        _ => actix_web::error::ErrorBadRequest(err),
                    })
                    .await
            })
        }
    }
}

/// Responder for GraphQL response
pub struct BatchGQLResponse(BatchQueryResponse);

impl From<BatchQueryResponse> for BatchGQLResponse {
    fn from(resp: BatchQueryResponse) -> Self {
        BatchGQLResponse(resp)
    }
}

impl Responder for BatchGQLResponse {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let mut res = HttpResponse::build(StatusCode::OK);
        res.content_type("application/json");
        add_cache_control_batch(&mut res, &self.0);
        let res = res.body(
            serde_json::to_string(&async_graphql::http::BatchGQLResponse::from(self.0)).unwrap(),
        );
        futures::future::ok(res)
    }
}

fn add_cache_control_batch(builder: &mut HttpResponseBuilder, resp: &BatchQueryResponse) {
    if let Some(cache_control) = resp.cache_control() {
        if let Some(cache_control) = cache_control.value() {
            builder.header("cache-control", cache_control);
        }
    }
}
