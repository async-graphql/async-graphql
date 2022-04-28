use std::{
    future::Future,
    io::{self, ErrorKind},
    pin::Pin,
};

use actix_http::{body::BoxBody, error::PayloadError};
use actix_web::{
    dev::Payload,
    error::JsonPayloadError,
    http,
    http::{Method, StatusCode},
    Error, FromRequest, HttpRequest, HttpResponse, Responder, Result,
};
use async_graphql::{http::MultipartOptions, ParseRequestError};
use futures_util::{
    future::{self, FutureExt},
    StreamExt, TryStreamExt,
};

/// Extractor for GraphQL request.
///
/// `async_graphql::http::MultipartOptions` allows to configure extraction
/// process.
pub struct GraphQLRequest(pub async_graphql::Request);

impl GraphQLRequest {
    /// Unwraps the value to `async_graphql::Request`.
    #[must_use]
    pub fn into_inner(self) -> async_graphql::Request {
        self.0
    }
}

type BatchToRequestMapper =
    fn(<<GraphQLBatchRequest as FromRequest>::Future as Future>::Output) -> Result<GraphQLRequest>;

impl FromRequest for GraphQLRequest {
    type Error = Error;
    type Future = future::Map<<GraphQLBatchRequest as FromRequest>::Future, BatchToRequestMapper>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        GraphQLBatchRequest::from_request(req, payload).map(|res| {
            Ok(Self(
                res?.0
                    .into_single()
                    .map_err(actix_web::error::ErrorBadRequest)?,
            ))
        })
    }
}

/// Extractor for GraphQL batch request.
///
/// `async_graphql::http::MultipartOptions` allows to configure extraction
/// process.
pub struct GraphQLBatchRequest(pub async_graphql::BatchRequest);

impl GraphQLBatchRequest {
    /// Unwraps the value to `async_graphql::BatchRequest`.
    #[must_use]
    pub fn into_inner(self) -> async_graphql::BatchRequest {
        self.0
    }
}

impl FromRequest for GraphQLBatchRequest {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<GraphQLBatchRequest>>>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let config = req
            .app_data::<MultipartOptions>()
            .cloned()
            .unwrap_or_default();

        if req.method() == Method::GET {
            let res = serde_urlencoded::from_str(req.query_string());
            Box::pin(async move { Ok(Self(async_graphql::BatchRequest::Single(res?))) })
        } else if req.method() == Method::POST {
            let content_type = req
                .headers()
                .get(http::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .map(|value| value.to_string());

            let (tx, rx) = async_channel::bounded(16);

            // Payload is !Send so we create indirection with a channel
            let mut payload = payload.take();
            actix::spawn(async move {
                while let Some(item) = payload.next().await {
                    if tx.send(item).await.is_err() {
                        return;
                    }
                }
            });

            Box::pin(async move {
                Ok(GraphQLBatchRequest(
                    async_graphql::http::receive_batch_body(
                        content_type,
                        rx.map_err(|e| match e {
                            PayloadError::Incomplete(Some(e)) | PayloadError::Io(e) => e,
                            PayloadError::Incomplete(None) => {
                                io::Error::from(ErrorKind::UnexpectedEof)
                            }
                            PayloadError::EncodingCorrupted => io::Error::new(
                                ErrorKind::InvalidData,
                                "cannot decode content-encoding",
                            ),
                            PayloadError::Overflow => io::Error::new(
                                ErrorKind::InvalidData,
                                "a payload reached size limit",
                            ),
                            PayloadError::UnknownLength => {
                                io::Error::new(ErrorKind::Other, "a payload length is unknown")
                            }
                            PayloadError::Http2Payload(e) if e.is_io() => e.into_io().unwrap(),
                            PayloadError::Http2Payload(e) => io::Error::new(ErrorKind::Other, e),
                            _ => io::Error::new(ErrorKind::Other, e),
                        })
                        .into_async_read(),
                        config,
                    )
                    .await
                    .map_err(|err| match err {
                        ParseRequestError::PayloadTooLarge => {
                            actix_web::error::ErrorPayloadTooLarge(err)
                        }
                        _ => actix_web::error::ErrorBadRequest(err),
                    })?,
                ))
            })
        } else {
            Box::pin(async move {
                Err(actix_web::error::ErrorMethodNotAllowed(
                    "GraphQL only supports GET and POST requests",
                ))
            })
        }
    }
}

/// Responder for a GraphQL response.
///
/// This contains a batch response, but since regular responses are a type of
/// batch response it works for both.
pub struct GraphQLResponse(pub async_graphql::BatchResponse);

impl From<async_graphql::Response> for GraphQLResponse {
    fn from(resp: async_graphql::Response) -> Self {
        Self(resp.into())
    }
}

impl From<async_graphql::BatchResponse> for GraphQLResponse {
    fn from(resp: async_graphql::BatchResponse) -> Self {
        Self(resp)
    }
}

#[cfg(feature = "cbor")]
mod cbor {
    use core::fmt;

    use actix_web::{http::StatusCode, ResponseError};

    #[derive(Debug)]
    pub struct Error(pub serde_cbor::Error);
    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl ResponseError for Error {
        fn status_code(&self) -> StatusCode {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

impl Responder for GraphQLResponse {
    type Body = BoxBody;

    fn respond_to(self, req: &HttpRequest) -> HttpResponse {
        let mut builder = HttpResponse::build(StatusCode::OK);

        if self.0.is_ok() {
            if let Some(cache_control) = self.0.cache_control().value() {
                builder.append_header((http::header::CACHE_CONTROL, cache_control));
            }
        }

        let accept = req
            .headers()
            .get(http::header::ACCEPT)
            .and_then(|val| val.to_str().ok());
        let (ct, body) = match accept {
            // optional cbor support
            #[cfg(feature = "cbor")]
            // this avoids copy-pasting the mime type
            Some(ct @ "application/cbor") => (
                ct,
                match serde_cbor::to_vec(&self.0) {
                    Ok(body) => body,
                    Err(e) => return HttpResponse::from_error(cbor::Error(e)),
                },
            ),
            _ => (
                "application/json",
                match serde_json::to_vec(&self.0) {
                    Ok(body) => body,
                    Err(e) => return HttpResponse::from_error(JsonPayloadError::Serialize(e)),
                },
            ),
        };

        let mut resp = builder.content_type(ct).body(body);

        for (name, value) in self.0.http_headers_iter() {
            resp.headers_mut().append(name, value);
        }

        resp
    }
}
