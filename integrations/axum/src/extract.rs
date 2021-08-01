use std::fmt::Display;
use std::io::ErrorKind;

use async_graphql::futures_util::TryStreamExt;
use async_graphql::http::MultipartOptions;
use async_graphql::ParseRequestError;
use axum::extract::{BodyStream, FromRequest, RequestParts};
use bytes::Buf;
use http::Method;
use tokio_util::compat::TokioAsyncReadCompatExt;

/// Extractor for GraphQL request.
pub struct GraphQLRequest(pub async_graphql::Request);

impl GraphQLRequest {
    /// Unwraps the value to `async_graphql::Request`.
    #[must_use]
    pub fn into_inner(self) -> async_graphql::Request {
        self.0
    }
}

/// Rejection response types.
pub mod rejection {
    use async_graphql::ParseRequestError;
    use axum::body::Body;
    use axum::response::IntoResponse;

    /// Rejection used for [`GraphQLRequest`](GraphQLRequest).
    pub struct GraphQLRejection(pub ParseRequestError);

    impl IntoResponse for GraphQLRejection {
        fn into_response(self) -> http::Response<Body> {
            todo!()
        }
    }

    impl From<ParseRequestError> for GraphQLRejection {
        fn from(err: ParseRequestError) -> Self {
            GraphQLRejection(err)
        }
    }
}

#[async_trait::async_trait]
impl<B> FromRequest<B> for GraphQLRequest
where
    B: http_body::Body + Unpin + Send + Sync + 'static,
    B::Data: Buf + Send,
    B::Error: Display + Send,
{
    type Rejection = rejection::GraphQLRejection;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        Ok(GraphQLRequest(
            GraphQLBatchRequest::from_request(req)
                .await?
                .0
                .into_single()?,
        ))
    }
}

/// Extractor for GraphQL batch request.
pub struct GraphQLBatchRequest(pub async_graphql::BatchRequest);

impl GraphQLBatchRequest {
    /// Unwraps the value to `async_graphql::BatchRequest`.
    #[must_use]
    pub fn into_inner(self) -> async_graphql::BatchRequest {
        self.0
    }
}

#[async_trait::async_trait]
impl<B> FromRequest<B> for GraphQLBatchRequest
where
    B: http_body::Body + Unpin + Send + Sync + 'static,
    B::Data: Buf + Send,
    B::Error: Display + Send,
{
    type Rejection = rejection::GraphQLRejection;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        if let (Some(&Method::GET), Some(uri)) = (req.method(), req.uri()) {
            let res = serde_urlencoded::from_str(uri.query().unwrap_or_default()).map_err(|err| {
                ParseRequestError::Io(std::io::Error::new(
                    ErrorKind::Other,
                    format!("failed to parse graphql request from uri query: {}", err),
                ))
            });
            Ok(Self(async_graphql::BatchRequest::Single(res?)))
        } else {
            let content_type = req
                .headers()
                .and_then(|headers| headers.get(http::header::CONTENT_TYPE))
                .and_then(|value| value.to_str().ok())
                .map(ToString::to_string);
            let body_stream = BodyStream::from_request(req)
                .await
                .map_err(|_| {
                    ParseRequestError::Io(std::io::Error::new(
                        ErrorKind::Other,
                        "body has been taken by another extractor".to_string(),
                    ))
                })?
                .map_err(|err| std::io::Error::new(ErrorKind::Other, err.to_string()));
            let body_reader = tokio_util::io::StreamReader::new(body_stream).compat();
            Ok(Self(
                async_graphql::http::receive_batch_body(
                    content_type,
                    body_reader,
                    MultipartOptions::default(),
                )
                .await?,
            ))
        }
    }
}
