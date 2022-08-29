//! A helper module that supports HTTP

mod graphiql_source;
mod graphiql_v2_source;
mod multipart;
mod playground_source;
mod websocket;

use futures_util::io::{AsyncRead, AsyncReadExt};
pub use graphiql_source::graphiql_source;
pub use graphiql_v2_source::GraphiQLSource;
use mime;
pub use multipart::MultipartOptions;
pub use playground_source::{playground_source, GraphQLPlaygroundConfig};
pub use websocket::{
    ClientMessage, Protocols as WebSocketProtocols, WebSocket, WsMessage, ALL_WEBSOCKET_PROTOCOLS,
};

use crate::{BatchRequest, ParseRequestError, Request};

/// Receive a GraphQL request from a content type and body.
pub async fn receive_body(
    content_type: Option<impl AsRef<str>>,
    body: impl AsyncRead + Send,
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
    // if no content-type header is set, we default to json
    let content_type = content_type
        .as_ref()
        .map(AsRef::as_ref)
        .unwrap_or("application/json");

    let content_type: mime::Mime = content_type.parse()?;

    match (content_type.type_(), content_type.subtype()) {
        // try to use multipart
        (mime::MULTIPART, _) => {
            if let Some(boundary) = content_type.get_param("boundary") {
                multipart::receive_batch_multipart(body, boundary.to_string(), opts).await
            } else {
                Err(ParseRequestError::InvalidMultipart(
                    multer::Error::NoBoundary,
                ))
            }
        }
        // application/json or cbor (currently)
        // cbor is in application/octet-stream.
        // Note: cbor will only match if feature ``cbor`` is active
        // TODO: wait for mime to add application/cbor and match against that too
        _ => receive_batch_body_no_multipart(&content_type, body).await,
    }
}

/// Receives a GraphQL query which is either cbor or json but NOT multipart
/// This method is only to avoid recursive calls with [``receive_batch_body``]
/// and [``multipart::receive_batch_multipart``]
pub(super) async fn receive_batch_body_no_multipart(
    content_type: &mime::Mime,
    body: impl AsyncRead + Send,
) -> Result<BatchRequest, ParseRequestError> {
    assert_ne!(content_type.type_(), mime::MULTIPART, "received multipart");
    match (content_type.type_(), content_type.subtype()) {
        #[cfg(feature = "cbor")]
        // cbor is in application/octet-stream.
        // TODO: wait for mime to add application/cbor and match against that too
        (mime::OCTET_STREAM, _) | (mime::APPLICATION, mime::OCTET_STREAM) => {
            receive_batch_cbor(body).await
        }
        // default to json
        _ => receive_batch_json(body).await,
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
    serde_json::from_slice::<BatchRequest>(&data)
        .map_err(|e| ParseRequestError::InvalidRequest(Box::new(e)))
}

/// Receive a GraphQL request from a body as CBOR.
#[cfg(feature = "cbor")]
#[cfg_attr(docsrs, doc(cfg(feature = "cbor")))]
pub async fn receive_cbor(body: impl AsyncRead) -> Result<Request, ParseRequestError> {
    receive_batch_cbor(body).await?.into_single()
}

/// Receive a GraphQL batch request from a body as CBOR
#[cfg(feature = "cbor")]
#[cfg_attr(docsrs, doc(cfg(feature = "cbor")))]
pub async fn receive_batch_cbor(body: impl AsyncRead) -> Result<BatchRequest, ParseRequestError> {
    let mut data = Vec::new();
    futures_util::pin_mut!(body);
    body.read_to_end(&mut data)
        .await
        .map_err(ParseRequestError::Io)?;
    serde_cbor::from_slice::<BatchRequest>(&data)
        .map_err(|e| ParseRequestError::InvalidRequest(Box::new(e)))
}
