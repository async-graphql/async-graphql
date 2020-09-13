//! A helper module that supports HTTP

mod graphiql_source;
mod multipart;
mod playground_source;
pub mod websocket;

pub use graphiql_source::graphiql_source;
pub use multipart::{receive_multipart, MultipartOptions};
pub use playground_source::{playground_source, GraphQLPlaygroundConfig};

use crate::{ParseRequestError, Request};
use futures::io::AsyncRead;
use futures::AsyncReadExt;

/// Receive a GraphQL request from a content type and body.
pub async fn receive_body(
    content_type: Option<impl AsRef<str>>,
    mut body: impl AsyncRead + Unpin + Send + 'static,
    opts: MultipartOptions,
) -> Result<Request, ParseRequestError> {
    if let Some(Ok(boundary)) = content_type.map(multer::parse_boundary) {
        receive_multipart(body, boundary, opts).await
    } else {
        let mut data = Vec::new();
        body.read_to_end(&mut data)
            .await
            .map_err(ParseRequestError::Io)?;
        Ok(serde_json::from_slice::<Request>(&data).map_err(ParseRequestError::InvalidRequest)?)
    }
}
