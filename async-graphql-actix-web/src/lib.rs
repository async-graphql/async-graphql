//! Async-graphql integration with Actix-web

#![warn(missing_docs)]
#![forbid(unsafe_code)]

mod subscription;

use actix_web::body::BodyStream;
use actix_web::dev::{HttpResponseBuilder, Payload, PayloadStream};
use actix_web::http::StatusCode;
use actix_web::{http, web, Error, FromRequest, HttpRequest, HttpResponse, Responder};
use async_graphql::http::{multipart_stream, StreamBody};
use async_graphql::{
    IntoQueryBuilder, IntoQueryBuilderOpts, ParseRequestError, QueryBuilder, QueryResponse,
    StreamResponse,
};
use futures::channel::mpsc;
use futures::future::Ready;
use futures::{Future, SinkExt, StreamExt, TryFutureExt};
use std::convert::Infallible;
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
    fn from(resp: async_graphql::Result<QueryResponse>) -> Self {
        GQLResponse(resp)
    }
}

impl Responder for GQLResponse {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let mut res = HttpResponse::build(StatusCode::OK);
        res.content_type("application/json");
        add_cache_control(&mut res, &self.0);
        let res =
            res.body(serde_json::to_string(&async_graphql::http::GQLResponse(self.0)).unwrap());
        futures::future::ok(res)
    }
}

/// Responder for GraphQL response stream
pub struct GQLResponseStream(StreamResponse);

impl From<StreamResponse> for GQLResponseStream {
    fn from(resp: StreamResponse) -> Self {
        GQLResponseStream(resp)
    }
}

impl Responder for GQLResponseStream {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, req: &HttpRequest) -> Self::Future {
        match self.0 {
            StreamResponse::Single(resp) => GQLResponse(resp).respond_to(req),
            StreamResponse::Stream(stream) => {
                let body =
                    BodyStream::new(multipart_stream(stream).map(Result::<_, Infallible>::Ok));
                let mut res = HttpResponse::build(StatusCode::OK);
                res.content_type("multipart/mixed; boundary=\"-\"");
                futures::future::ok(res.body(body))
            }
        }
    }
}

fn add_cache_control(
    builder: &mut HttpResponseBuilder,
    resp: &async_graphql::Result<QueryResponse>,
) {
    if let Ok(QueryResponse { cache_control, .. }) = resp {
        if let Some(cache_control) = cache_control.value() {
            builder.header("cache-control", cache_control);
        }
    }
}
