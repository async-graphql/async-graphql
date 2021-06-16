//! A helper module that supports HTTP

mod graphiql_source;
mod multipart;
mod playground_source;
mod websocket;

use futures_util::io::{AsyncRead, AsyncReadExt};

use crate::{BatchRequest, ParseRequestError, Request};

pub use graphiql_source::graphiql_source;
pub use multipart::MultipartOptions;
pub use playground_source::{playground_source, GraphQLPlaygroundConfig};
pub use websocket::{ClientMessage, Protocols as WebSocketProtocols, WebSocket, WsMessage};

/// Receive a GraphQL request from a content type and body.
pub async fn receive_body(
    content_type: Option<impl AsRef<str>>,
    body: impl AsyncRead + Send + 'static,
    opts: MultipartOptions,
) -> Result<Request, ParseRequestError> {
    receive_batch_body(content_type, body, opts)
        .await?
        .into_single()
}

/// Receive a GraphQL request from a content type and body.
pub async fn receive_batch_body(
    content_type: Option<impl AsRef<str>>,
    body: impl AsyncRead + Send,
    opts: MultipartOptions,
) -> Result<BatchRequest, ParseRequestError> {
    let content_type = content_type.as_ref().map(AsRef::as_ref);

    if let Some(Ok(boundary)) = content_type.map(multer::parse_boundary) {
        multipart::receive_batch_multipart(body, boundary, opts).await
    } else {
        receive_batch_json(body).await
    }
}

/// Receive a GraphQL request from a body as JSON.
pub async fn receive_json(body: impl AsyncRead) -> Result<Request, ParseRequestError> {
    receive_batch_json(body).await?.into_single()
}

/// Receive a GraphQL batch request from a body as JSON.
pub async fn receive_batch_json(body: impl AsyncRead) -> Result<BatchRequest, ParseRequestError> {
    let mut data = Vec::new();
    futures_util::pin_mut!(body);
    body.read_to_end(&mut data)
        .await
        .map_err(ParseRequestError::Io)?;
    Ok(serde_json::from_slice::<BatchRequest>(&data).map_err(ParseRequestError::InvalidRequest)?)
}
