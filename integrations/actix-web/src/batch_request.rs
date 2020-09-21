use actix_web::dev::{HttpResponseBuilder, Payload, PayloadStream};
use actix_web::http::StatusCode;
use actix_web::{http, web, Error, FromRequest, HttpRequest, HttpResponse, Responder};
use async_graphql::http::MultipartOptions;
use async_graphql::ParseRequestError;
use futures::channel::mpsc;
use futures::future::Ready;
use futures::io::ErrorKind;
use futures::{Future, SinkExt, StreamExt, TryFutureExt, TryStreamExt};
use std::io;
use std::pin::Pin;

/// Extractor for GraphQL batch request
///
/// It's a wrapper of `async_graphql::Request`, you can use `Request::into_inner` unwrap it to `async_graphql::Request`.
/// `async_graphql::http::MultipartOptions` allows to configure extraction process.
pub struct BatchRequest(async_graphql::BatchRequest);

impl BatchRequest {
    /// Unwraps the value to `async_graphql::Request`.
    pub fn into_inner(self) -> async_graphql::BatchRequest {
        self.0
    }
}

impl FromRequest for BatchRequest {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<BatchRequest, Error>>>>;
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

/// Responder for GraphQL batch response
pub struct BatchResponse(async_graphql::BatchResponse);

impl From<async_graphql::BatchResponse> for BatchResponse {
    fn from(resp: async_graphql::BatchResponse) -> Self {
        BatchResponse(resp)
    }
}

impl Responder for BatchResponse {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let mut res = HttpResponse::build(StatusCode::OK);
        res.content_type("application/json");
        add_cache_control(&mut res, &self.0);
        let res = res.body(serde_json::to_string(&self.0).unwrap());
        futures::future::ok(res)
    }
}

fn add_cache_control(builder: &mut HttpResponseBuilder, resp: &async_graphql::BatchResponse) {
    if resp.is_ok() {
        if let Some(cache_control) = resp.cache_control().value() {
            builder.header("cache-control", cache_control);
        }
    }
}
