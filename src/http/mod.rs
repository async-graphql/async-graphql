//! A helper module that supports HTTP

mod graphiql_source;
mod playground_source;
mod stream_body;

use itertools::Itertools;

pub use graphiql_source::graphiql_source;
pub use playground_source::{playground_source, GraphQLPlaygroundConfig};
pub use stream_body::StreamBody;

use crate::{Error, GQLQueryResponse, ParseRequestError, Pos, QueryError, Result, Variables};
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Serialize, Serializer};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pos;
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
