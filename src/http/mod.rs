//! A helper module that supports HTTP

mod graphiql_source;
mod multipart;
mod playground_source;
mod stream_body;

pub use graphiql_source::graphiql_source;
pub use multipart::{receive_multipart, MultipartOptions};
pub use playground_source::{playground_source, GraphQLPlaygroundConfig};
pub use stream_body::StreamBody;

use crate::{Data, ParseRequestError, Pos, QueryError, Request, Variables};
use futures::io::AsyncRead;
use futures::AsyncReadExt;
use serde::ser::{SerializeMap, SerializeSeq};
use serde::Deserialize;

/// Deserializable GraphQL Request object
#[derive(Deserialize, Clone, PartialEq, Debug)]
pub struct GQLRequest {
    /// Query source
    pub query: String,

    /// Operation name for this query
    #[serde(rename = "operationName")]
    pub operation_name: Option<String>,

    /// Variables for this query
    pub variables: Option<serde_json::Value>,
}

impl From<GQLRequest> for Request {
    fn from(request: GQLRequest) -> Self {
        Self {
            query: request.query,
            operation_name: request.operation_name,
            variables: request
                .variables
                .map(|value| Variables::parse_from_json(value))
                .unwrap_or_default(),
            ctx_data: Data::default(),
        }
    }
}

/// Receive a GraphQL request from a content type and body.
pub async fn receive_body(
    content_type: Option<impl AsRef<str>>,
    mut body: impl AsyncRead + Send + Unpin + 'static,
    opts: MultipartOptions,
) -> Result<Request, ParseRequestError> {
    if let Some(Ok(boundary)) = content_type.map(multer::parse_boundary) {
        receive_multipart(body, boundary, opts).await
    } else {
        let mut data = Vec::new();
        body.read_to_end(&mut data)
            .await
            .map_err(ParseRequestError::Io)?;
        Ok(serde_json::from_slice::<GQLRequest>(&data)
            .map_err(ParseRequestError::InvalidRequest)?
            .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_request() {
        let request: GQLRequest = serde_json::from_value(json! ({
            "query": "{ a b c }"
        }))
        .unwrap();
        assert!(request.variables.is_none());
        assert!(request.operation_name.is_none());
        assert_eq!(request.query, "{ a b c }");
    }

    #[test]
    fn test_request_with_operation_name() {
        let request: GQLRequest = serde_json::from_value(json! ({
            "query": "{ a b c }",
            "operationName": "a"
        }))
        .unwrap();
        assert!(request.variables.is_none());
        assert_eq!(request.operation_name.as_deref(), Some("a"));
        assert_eq!(request.query, "{ a b c }");
    }

    #[test]
    fn test_request_with_variables() {
        let request: GQLRequest = serde_json::from_value(json! ({
            "query": "{ a b c }",
            "variables": {
                "v1": 100,
                "v2": [1, 2, 3],
                "v3": "str",
            }
        }))
        .unwrap();
        assert_eq!(
            request.variables,
            Some(json!({
                "v1": 100,
                "v2": [1, 2, 3],
                "v3": "str",
            }))
        );
        assert!(request.operation_name.is_none());
        assert_eq!(request.query, "{ a b c }");
    }
}
