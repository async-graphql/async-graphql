//! A helper module that supports HTTP

#[cfg(feature = "graphiql")]
mod graphiql_plugin;
#[cfg(feature = "graphiql")]
mod graphiql_source;
#[cfg(feature = "graphiql")]
mod graphiql_v2_source;
mod multipart;
mod multipart_subscribe;
#[cfg(feature = "playground")]
mod playground_source;
mod websocket;

use std::io::ErrorKind;

use futures_util::io::{AsyncRead, AsyncReadExt};
#[cfg(feature = "graphiql")]
pub use graphiql_plugin::{graphiql_plugin_explorer, GraphiQLPlugin};
#[cfg(feature = "graphiql")]
pub use graphiql_source::graphiql_source;
#[cfg(feature = "graphiql")]
pub use graphiql_v2_source::{Credentials, GraphiQLSource};
use mime;
pub use multipart::MultipartOptions;
pub use multipart_subscribe::{create_multipart_mixed_stream, is_accept_multipart_mixed};
#[cfg(feature = "playground")]
pub use playground_source::{playground_source, GraphQLPlaygroundConfig};
use serde::Deserialize;
pub use websocket::{
    ClientMessage, Protocols as WebSocketProtocols, WebSocket, WsMessage, ALL_WEBSOCKET_PROTOCOLS,
};

use crate::{BatchRequest, ParseRequestError, Request};

/// Parse a GraphQL request from a query string.
pub fn parse_query_string(input: &str) -> Result<Request, ParseRequestError> {
    #[derive(Deserialize)]
    struct RequestSerde {
        #[serde(default)]
        pub query: String,
        pub operation_name: Option<String>,
        pub variables: Option<String>,
        pub extensions: Option<String>,
    }

    let request: RequestSerde = serde_urlencoded::from_str(input)
        .map_err(|err| std::io::Error::new(ErrorKind::Other, err))?;
    let variables = request
        .variables
        .map(|data| serde_json::from_str(&data))
        .transpose()
        .map_err(|err| {
            std::io::Error::new(ErrorKind::Other, format!("invalid variables: {}", err))
        })?
        .unwrap_or_default();
    let extensions = request
        .extensions
        .map(|data| serde_json::from_str(&data))
        .transpose()
        .map_err(|err| {
            std::io::Error::new(ErrorKind::Other, format!("invalid extensions: {}", err))
        })?
        .unwrap_or_default();

    Ok(Request {
        operation_name: request.operation_name,
        variables,
        extensions,
        ..Request::new(request.query)
    })
}

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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::{value, Variables};

    #[test]
    fn test_parse_query_string() {
        let request = parse_query_string("variables=%7B%7D&extensions=%7B%22persistedQuery%22%3A%7B%22sha256Hash%22%3A%22cde5de0a350a19c59f8ddcd9646e5f260b2a7d5649ff6be8e63e9462934542c3%22%2C%22version%22%3A1%7D%7D").unwrap();
        assert_eq!(request.query.as_str(), "");
        assert_eq!(request.variables, Variables::default());
        assert_eq!(request.extensions, {
            let mut extensions = HashMap::new();
            extensions.insert("persistedQuery".to_string(), value!({
                "sha256Hash": "cde5de0a350a19c59f8ddcd9646e5f260b2a7d5649ff6be8e63e9462934542c3",
                "version": 1,
            }));
            extensions
        });

        let request = parse_query_string("query={a}&variables=%7B%22a%22%3A10%7D").unwrap();
        assert_eq!(request.query.as_str(), "{a}");
        assert_eq!(
            request.variables,
            Variables::from_value(value!({ "a" : 10 }))
        );
    }
}
