//! A helper module that supports HTTP

mod graphiql_source;
#[cfg(feature = "multipart")]
mod multipart;
mod playground_source;
pub mod websocket;

pub use graphiql_source::graphiql_source;
#[cfg(feature = "multipart")]
pub use multipart::{receive_multipart, MultipartOptions};
pub use playground_source::{playground_source, GraphQLPlaygroundConfig};

use crate::{ParseRequestError, Request};
use futures::io::AsyncRead;
use futures::AsyncReadExt;

/// Receive a GraphQL request from a content type and body.
///
/// If the content type is multipart it will use `receive_multipart`, otherwise it will use
/// `receive_json`.
#[cfg(feature = "multipart")]
pub async fn receive_body(
    content_type: Option<impl AsRef<str>>,
    body: impl AsyncRead + Send + 'static,
    opts: MultipartOptions,
) -> Result<Request, ParseRequestError> {
    if let Some(Ok(boundary)) = content_type.map(multer::parse_boundary) {
        receive_multipart(body, boundary, opts).await
    } else {
        receive_json(body).await
    }
}

/// Receive a GraphQL request from a body as JSON.
pub async fn receive_json(
    body: impl AsyncRead + Send + 'static,
) -> Result<Request, ParseRequestError> {
    let mut data = Vec::new();
    futures::pin_mut!(body);
    body.read_to_end(&mut data)
        .await
        .map_err(ParseRequestError::Io)?;
    Ok(serde_json::from_slice::<Request>(&data).map_err(ParseRequestError::InvalidRequest)?)
}
