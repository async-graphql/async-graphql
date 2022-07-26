//! A helper module that supports HTTP

mod graphiql_source;
mod multipart;
mod playground_source;
mod websocket;

use futures_util::io::{AsyncRead, AsyncReadExt};
pub use graphiql_source::graphiql_source;
use mime;
pub use multipart::MultipartOptions;
pub use playground_source::{playground_source, GraphQLPlaygroundConfig};
pub use websocket::{
    ClientMessage, Protocols as WebSocketProtocols, WebSocket, WsMessage, ALL_WEBSOCKET_PROTOCOLS,
};

use crate::{BatchRequest, ParseRequestError, Request};

/// supported content-encoding values
#[derive(Debug, Clone, Copy)]
pub enum ContentEncoding {
    /// LZ77
    Gzip,

    /// Deflate
    Deflate,

    /// Brotli
    Br,

    /// Zstd
    Zstd,
}

/// Receive a GraphQL request from a content type and body.
pub async fn receive_body(
    content_type: Option<impl AsRef<str>>,
    content_encoding: Option<impl AsRef<str>>,
    body: impl AsyncRead + Send,
    opts: MultipartOptions,
) -> Result<Request, ParseRequestError> {
    receive_batch_body(content_type, content_encoding, body, opts)
        .await?
        .into_single()
}

/// Receive a GraphQL request from a content type and body.
pub async fn receive_batch_body(
    content_type: Option<impl AsRef<str>>,
    content_encoding: Option<impl AsRef<str>>,
    body: impl AsyncRead + Send,
    opts: MultipartOptions,
) -> Result<BatchRequest, ParseRequestError> {
    // if no content-type header is set, we default to json
    let content_type = content_type
        .as_ref()
        .map(AsRef::as_ref)
        .unwrap_or("application/json");

    // parse the content-encoding
    let content_encoding = match content_encoding.as_ref().map(AsRef::as_ref) {
        Some("gzip") => Some(ContentEncoding::Gzip),
        Some("deflate") => Some(ContentEncoding::Deflate),
        Some("br") => Some(ContentEncoding::Br),
        Some("zstd") => Some(ContentEncoding::Zstd),
        _ => None,
    };

    let content_type: mime::Mime = content_type.parse()?;

    match (content_type.type_(), content_type.subtype()) {
        // try to use multipart
        (mime::MULTIPART, _) => {
            if let Some(boundary) = content_type.get_param("boundary") {
                multipart::receive_batch_multipart(
                    body,
                    content_encoding,
                    boundary.to_string(),
                    opts,
                )
                .await
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
        _ => receive_batch_body_no_multipart(&content_type, content_encoding, body).await,
    }
}

/// Receives a GraphQL query which is either cbor or json but NOT multipart
/// This method is only to avoid recursive calls with [``receive_batch_body``]
/// and [``multipart::receive_batch_multipart``]
pub(super) async fn receive_batch_body_no_multipart(
    content_type: &mime::Mime,
    content_encoding: Option<ContentEncoding>,
    body: impl AsyncRead + Send,
) -> Result<BatchRequest, ParseRequestError> {
    assert_ne!(content_type.type_(), mime::MULTIPART, "received multipart");
    match (content_type.type_(), content_type.subtype()) {
        #[cfg(feature = "cbor")]
        // cbor is in application/octet-stream.
        // TODO: wait for mime to add application/cbor and match against that too
        (mime::OCTET_STREAM, _) | (mime::APPLICATION, mime::OCTET_STREAM) => {
            receive_batch_cbor(body, content_encoding).await
        }
        // default to json
        _ => receive_batch_json(body, content_encoding).await,
    }
}
/// Receive a GraphQL request from a body as JSON.
pub async fn receive_json(
    body: impl AsyncRead,
    content_encoding: Option<ContentEncoding>,
) -> Result<Request, ParseRequestError> {
    receive_batch_json(body, content_encoding)
        .await?
        .into_single()
}

/// Receive a GraphQL batch request from a body as JSON.
pub async fn receive_batch_json(
    body: impl AsyncRead,
    content_encoding: Option<ContentEncoding>,
) -> Result<BatchRequest, ParseRequestError> {
    let mut data = Vec::new();
    futures_util::pin_mut!(body);

    body.read_to_end(&mut data)
        .await
        .map_err(ParseRequestError::Io)?;

    data = handle_content_encoding(data, content_encoding)?;

    serde_json::from_slice::<BatchRequest>(&data)
        .map_err(|e| ParseRequestError::InvalidRequest(Box::new(e)))
}

/// Receive a GraphQL request from a body as CBOR.
#[cfg(feature = "cbor")]
#[cfg_attr(docsrs, doc(cfg(feature = "cbor")))]
pub async fn receive_cbor(
    body: impl AsyncRead,
    content_encoding: Option<ContentEncoding>,
) -> Result<Request, ParseRequestError> {
    receive_batch_cbor(body, content_encoding)
        .await?
        .into_single()
}

/// Receive a GraphQL batch request from a body as CBOR
#[cfg(feature = "cbor")]
#[cfg_attr(docsrs, doc(cfg(feature = "cbor")))]
pub async fn receive_batch_cbor(
    body: impl AsyncRead,
    content_encoding: Option<ContentEncoding>,
) -> Result<BatchRequest, ParseRequestError> {
    let mut data = Vec::new();
    futures_util::pin_mut!(body);
    body.read_to_end(&mut data)
        .await
        .map_err(ParseRequestError::Io)?;

    data = handle_content_encoding(data, content_encoding)?;

    serde_cbor::from_slice::<BatchRequest>(&data)
        .map_err(|e| ParseRequestError::InvalidRequest(Box::new(e)))
}

/// decompress data if needed
#[cfg(not(feature = "compression"))]
fn handle_content_encoding(
    data: Vec<u8>,
    _: Option<ContentEncoding>,
) -> Result<Vec<u8>, ParseRequestError> {
    Ok(data)
}

/// decompress data if needed
#[cfg(feature = "compression")]
fn handle_content_encoding(
    data: Vec<u8>,
    content_encoding: Option<ContentEncoding>,
) -> Result<Vec<u8>, ParseRequestError> {
    use std::io::prelude::*;

    use flate2::read::{GzDecoder, ZlibDecoder};

    match content_encoding {
        Some(ContentEncoding::Gzip) => {
            let mut buff = Vec::new();
            GzDecoder::new(data.as_slice())
                .read_to_end(&mut buff)
                .map_err(ParseRequestError::Io)?;
            Ok(buff)
        }
        Some(ContentEncoding::Deflate) => {
            let mut buff = Vec::new();
            ZlibDecoder::new(data.as_slice())
                .read_to_end(&mut buff)
                .map_err(ParseRequestError::Io)?;
            Ok(buff)
        }
        Some(ContentEncoding::Br) => {
            let mut buff = Vec::new();
            brotli::Decompressor::new(data.as_slice(), 8192)
                .read_to_end(&mut buff)
                .map_err(ParseRequestError::Io)?;
            Ok(buff)
        }
        Some(ContentEncoding::Zstd) => {
            let mut buff = Vec::new();
            zstd::stream::copy_decode(data.as_slice(), &mut buff).map_err(ParseRequestError::Io)?;
            Ok(buff)
        }
        None => Ok(data),
    }
}
