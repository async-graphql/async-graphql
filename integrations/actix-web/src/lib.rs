//! Async-graphql integration with Actix-web
#![forbid(unsafe_code)]

mod subscription;

pub use subscription::WSSubscription;

use actix_web::dev::{Payload, PayloadStream};
use actix_web::http::StatusCode;
use actix_web::{http, web, Error, FromRequest, HttpRequest, HttpResponse, Responder, Result};
use async_graphql::http::MultipartOptions;
use async_graphql::ParseRequestError;
use futures::channel::mpsc;
use futures::future::{self, FutureExt, Ready};
use futures::io::ErrorKind;
use futures::{Future, SinkExt, StreamExt, TryFutureExt, TryStreamExt};
use std::io;
use std::pin::Pin;

/// Extractor for GraphQL request.
///
/// `async_graphql::http::MultipartOptions` allows to configure extraction process.
pub struct Request(pub async_graphql::Request);

impl Request {
    /// Unwraps the value to `async_graphql::Request`.
    #[must_use]
    pub fn into_inner(self) -> async_graphql::Request {
        self.0
    }
}

type RequestMapper =
    fn(<<BatchRequest as FromRequest>::Future as Future>::Output) -> Result<Request>;

impl FromRequest for Request {
    type Error = Error;
    type Future = future::Map<<BatchRequest as FromRequest>::Future, RequestMapper>;
    type Config = MultipartOptions;

    fn from_request(req: &HttpRequest, payload: &mut Payload<PayloadStream>) -> Self::Future {
        BatchRequest::from_request(req, payload).map(|res| {
            res.and_then(|batch| {
                batch
                    .0
                    .into_single()
                    .map_err(actix_web::error::ErrorBadRequest)
            })
            .map(Self)
        })
    }
}

/// Extractor for GraphQL batch request.
///
/// `async_graphql::http::MultipartOptions` allows to configure extraction process.
pub struct BatchRequest(pub async_graphql::BatchRequest);

impl BatchRequest {
    /// Unwraps the value to `async_graphql::BatchRequest`.
    #[must_use]
    pub fn into_inner(self) -> async_graphql::BatchRequest {
        self.0
    }
}

impl FromRequest for BatchRequest {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<BatchRequest>>>>;
    type Config = MultipartOptions;

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
            Ok(BatchRequest(
                async_graphql::http::receive_batch_body(
                    content_type,
                    rx.map_err(|err| io::Error::new(ErrorKind::Other, err))
                        .into_async_read(),
                    config,
                )
                .map_err(|err| match err {
                    ParseRequestError::PayloadTooLarge => {
                        actix_web::error::ErrorPayloadTooLarge(err)
                    }
                    _ => actix_web::error::ErrorBadRequest(err),
                })
                .await?,
            ))
        })
    }
}

/// Responder for a GraphQL response.
///
/// This contains a batch response, but since regular responses are a type of batch response it
/// works for both.
pub struct Response(pub async_graphql::BatchResponse);

impl From<async_graphql::Response> for Response {
    fn from(resp: async_graphql::Response) -> Self {
        Self(resp.into())
    }
}

impl From<async_graphql::BatchResponse> for Response {
    fn from(resp: async_graphql::BatchResponse) -> Self {
        Self(resp)
    }
}

impl Responder for Response {
    type Error = Error;
    type Future = Ready<Result<HttpResponse>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let mut res = HttpResponse::build(StatusCode::OK);
        res.content_type("application/json");
        if self.0.is_ok() {
            if let Some(cache_control) = self.0.cache_control().value() {
                res.header("cache-control", cache_control);
            }
        }
        futures::future::ok(res.body(serde_json::to_string(&self.0).unwrap()))
    }
}
