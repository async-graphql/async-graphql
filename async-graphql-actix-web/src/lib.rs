//! Async-graphql integration with Actix-web

#![warn(missing_docs)]

mod subscription;

use actix_web::body::BodyStream;
use actix_web::dev::{Payload, PayloadStream};
use actix_web::http::StatusCode;
use actix_web::{http, web, Error, FromRequest, HttpRequest, HttpResponse, Responder};
use async_graphql::http::StreamBody;
use async_graphql::{
    IntoQueryBuilder, IntoQueryBuilderOpts, ParseRequestError, QueryBuilder, QueryResponse,
};
use bytes::{buf::BufExt, Buf, Bytes};
use futures::channel::mpsc;
use futures::future::Ready;
use futures::{Future, SinkExt, Stream, StreamExt, TryFutureExt};
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

/// Responder for GraphQL response
pub struct GQLResponse(async_graphql::Result<QueryResponse>);

impl From<async_graphql::Result<QueryResponse>> for GQLResponse {
    fn from(res: async_graphql::Result<QueryResponse>) -> Self {
        GQLResponse(res)
    }
}

impl Responder for GQLResponse {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let res = HttpResponse::build(StatusCode::OK)
            .content_type("application/json")
            .body(serde_json::to_string(&async_graphql::http::GQLResponse(self.0)).unwrap());
        futures::future::ok(res)
    }
}

/// Responder for GraphQL response stream
pub struct GQLResponseStream<S: Stream<Item = async_graphql::Result<QueryResponse>>>(S);

impl<S: Stream<Item = async_graphql::Result<QueryResponse>> + 'static> From<S>
    for GQLResponseStream<S>
{
    fn from(stream: S) -> Self {
        GQLResponseStream(stream)
    }
}

impl<S: Stream<Item = async_graphql::Result<QueryResponse>> + Unpin + 'static> Responder
    for GQLResponseStream<S>
{
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = BodyStream::new(
            self.0
                .map(|res| serde_json::to_vec(&async_graphql::http::GQLResponse(res)).unwrap())
                .map(|data| {
                    Ok::<_, std::convert::Infallible>(
                        Bytes::from(format!(
                            "\r\n---\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
                            data.len()
                        ))
                        .chain(Bytes::from(data))
                        .to_bytes(),
                    )
                })
                .chain(futures::stream::once(futures::future::ok(
                    Bytes::from_static(b"\r\n-----\r\n"),
                ))),
        );
        let res = HttpResponse::build(StatusCode::OK)
            .content_type("multipart/mixed; boundary=\"-\"")
            .body(body);
        futures::future::ok(res)
    }
}
