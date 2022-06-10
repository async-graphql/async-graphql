use std::{io::ErrorKind, marker::PhantomData};

use async_graphql::{futures_util::TryStreamExt, http::MultipartOptions, ParseRequestError};
use axum::{
    extract::{BodyStream, FromRequest, RequestParts},
    http,
    http::Method,
    response::IntoResponse,
    BoxError,
};
use bytes::Bytes;
use tokio_util::compat::TokioAsyncReadCompatExt;

/// Extractor for GraphQL request.
pub struct GraphQLRequest<R = rejection::GraphQLRejection>(
    pub async_graphql::Request,
    PhantomData<R>,
);

impl<R> GraphQLRequest<R> {
    /// Unwraps the value to `async_graphql::Request`.
    #[must_use]
    pub fn into_inner(self) -> async_graphql::Request {
        self.0
    }
}

/// Rejection response types.
pub mod rejection {
    use async_graphql::ParseRequestError;
    use axum::{
        body::{boxed, Body, BoxBody},
        http,
        http::StatusCode,
        response::IntoResponse,
    };

    /// Rejection used for [`GraphQLRequest`](GraphQLRequest).
    pub struct GraphQLRejection(pub ParseRequestError);

    impl IntoResponse for GraphQLRejection {
        fn into_response(self) -> http::Response<BoxBody> {
            match self.0 {
                ParseRequestError::PayloadTooLarge => http::Response::builder()
                    .status(StatusCode::PAYLOAD_TOO_LARGE)
                    .body(boxed(Body::empty()))
                    .unwrap(),
                bad_request => http::Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(boxed(Body::from(format!("{:?}", bad_request))))
                    .unwrap(),
            }
        }
    }

    impl From<ParseRequestError> for GraphQLRejection {
        fn from(err: ParseRequestError) -> Self {
            GraphQLRejection(err)
        }
    }
}

#[async_trait::async_trait]
impl<B, R> FromRequest<B> for GraphQLRequest<R>
where
    B: http_body::Body + Unpin + Send + Sync + 'static,
    B::Data: Into<Bytes>,
    B::Error: Into<BoxError>,
    R: IntoResponse + From<ParseRequestError>,
{
    type Rejection = R;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        Ok(GraphQLRequest(
            GraphQLBatchRequest::<R>::from_request(req)
                .await?
                .0
                .into_single()?,
            PhantomData,
        ))
    }
}

/// Extractor for GraphQL batch request.
pub struct GraphQLBatchRequest<R = rejection::GraphQLRejection>(
    pub async_graphql::BatchRequest,
    PhantomData<R>,
);

impl<R> GraphQLBatchRequest<R> {
    /// Unwraps the value to `async_graphql::BatchRequest`.
    #[must_use]
    pub fn into_inner(self) -> async_graphql::BatchRequest {
        self.0
    }
}

#[async_trait::async_trait]
impl<B, R> FromRequest<B> for GraphQLBatchRequest<R>
where
    B: http_body::Body + Unpin + Send + Sync + 'static,
    B::Data: Into<Bytes>,
    B::Error: Into<BoxError>,
    R: IntoResponse + From<ParseRequestError>,
{
    type Rejection = R;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        if let (&Method::GET, uri) = (req.method(), req.uri()) {
            let res = serde_urlencoded::from_str(uri.query().unwrap_or_default()).map_err(|err| {
                ParseRequestError::Io(std::io::Error::new(
                    ErrorKind::Other,
                    format!("failed to parse graphql request from uri query: {}", err),
                ))
            });
            Ok(Self(async_graphql::BatchRequest::Single(res?), PhantomData))
        } else {
            let content_type = req
                .headers()
                .get(http::header::CONTENT_TYPE)
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
                PhantomData,
            ))
        }
    }
}
