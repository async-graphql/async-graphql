//! A helper module that supports HTTP

mod graphiql_source;
mod into_query_builder;
mod playground_source;
mod stream_body;

use itertools::Itertools;

pub use graphiql_source::graphiql_source;
pub use playground_source::{playground_source, GraphQLPlaygroundConfig};
pub use stream_body::StreamBody;

use crate::query::{
    IntoBatchQueryDefinition, IntoQueryBuilderOpts, QueryDefinition, QueryDefinitionTypes,
};
use crate::{
    BatchQueryDefinition, BatchQueryResponse, Error, ParseRequestError, Pos, QueryError,
    QueryResponse, Result, Variables,
};
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{de, Deserialize, Serialize, Serializer};

/// Deserializable GraphQL Request object
#[derive(Deserialize, Clone, PartialEq, Debug)]
pub struct GQLRequestPart {
    /// Query source
    pub query: String,

    /// Operation name for this query
    #[serde(rename = "operationName")]
    pub operation_name: Option<String>,

    /// Variables for this query
    pub variables: Option<serde_json::Value>,
}

impl From<GQLRequestPart> for QueryDefinition {
    fn from(request: GQLRequestPart) -> Self {
        Self {
            query_source: request.query,
            operation_name: request.operation_name,
            // Unwrap twice as we use default variables if no variables provided, and if serde
            // fails to deserialize them
            variables: request
                .variables
                .map(Variables::parse_from_json)
                .unwrap_or(Ok(Variables::default()))
                .unwrap_or_default(),
            extensions: vec![],
        }
    }
}

/// Batch support for GraphQL requests, which is either a single query, or an array of queries
#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(untagged)]
pub enum BatchGQLRequest {
    /// Single query
    Single(GQLRequestPart),
    /// Non-empty array of queries
    #[serde(deserialize_with = "deserialize_non_empty_vec")]
    Batch(Vec<GQLRequestPart>),
}

fn deserialize_non_empty_vec<'de, D, T>(deserializer: D) -> std::result::Result<Vec<T>, D::Error>
where
    D: de::Deserializer<'de>,
    T: Deserialize<'de>,
{
    use de::Error as _;

    let v = Vec::<T>::deserialize(deserializer)?;
    if v.is_empty() {
        Err(D::Error::invalid_length(0, &"a positive integer"))
    } else {
        Ok(v)
    }
}

#[async_trait::async_trait]
impl IntoBatchQueryDefinition for BatchGQLRequest {
    async fn into_batch_query_definition_opts(
        self,
        _opts: &IntoQueryBuilderOpts,
    ) -> std::result::Result<BatchQueryDefinition, ParseRequestError> {
        match self {
            BatchGQLRequest::Single(request) => Ok(BatchQueryDefinition {
                definition: QueryDefinitionTypes::Single(request.into()),
                ctx_data: None,
            }),
            BatchGQLRequest::Batch(requests) => {
                let definitions = requests.into_iter().map(|request| request.into()).collect();
                Ok(BatchQueryDefinition {
                    definition: QueryDefinitionTypes::Batch(definitions),
                    ctx_data: None,
                })
            }
        }
    }
}

/// Serializable GraphQL Response object
pub struct GQLResponse(pub Result<QueryResponse>);

impl Serialize for GQLResponse {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match &self.0 {
            Ok(res) => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_key("data")?;
                map.serialize_value(&res.data)?;
                if res.extensions.is_some() {
                    map.serialize_key("extensions")?;
                    map.serialize_value(&res.extensions)?;
                }
                map.end()
            }
            Err(err) => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_key("errors")?;
                map.serialize_value(&GQLError(err))?;
                map.end()
            }
        }
    }
}

/// Serializable GraphQL Response object for batchable queries
#[derive(Serialize)]
#[serde(untagged)]
pub enum BatchGQLResponse {
    /// Response for single queries
    Single(GQLResponse),
    /// Response for batch queries
    Batch(Vec<GQLResponse>),
}

impl From<BatchQueryResponse> for BatchGQLResponse {
    fn from(item: BatchQueryResponse) -> Self {
        match item {
            BatchQueryResponse::Single(resp) => BatchGQLResponse::Single(GQLResponse(resp)),
            BatchQueryResponse::Batch(responses) => {
                BatchGQLResponse::Batch(responses.into_iter().map(GQLResponse).collect())
            }
        }
    }
}

/// Serializable error type
pub struct GQLError<'a>(pub &'a Error);

impl<'a> Serialize for GQLError<'a> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.0 {
            Error::Parse(err) => {
                let mut seq = serializer.serialize_seq(Some(1))?;
                seq.serialize_element(&serde_json::json! ({
                    "message": err.message,
                    "locations": [{"line": err.pos.line, "column": err.pos.column}]
                }))?;
                seq.end()
            }
            Error::Query { pos, path, err } => {
                let mut seq = serializer.serialize_seq(Some(1))?;
                if let QueryError::FieldError {
                    err,
                    extended_error,
                } = err
                {
                    let mut map = serde_json::Map::new();

                    map.insert("message".to_string(), err.to_string().into());
                    map.insert(
                        "locations".to_string(),
                        serde_json::json!([{"line": pos.line, "column": pos.column}]),
                    );

                    if let Some(path) = path {
                        map.insert("path".to_string(), path.clone());
                    }

                    if let Some(obj @ serde_json::Value::Object(_)) = extended_error {
                        map.insert("extensions".to_string(), obj.clone());
                    }

                    seq.serialize_element(&serde_json::Value::Object(map))?;
                } else {
                    seq.serialize_element(&serde_json::json!({
                        "message": err.to_string(),
                        "locations": [{"line": pos.line, "column": pos.column}]
                    }))?;
                }
                seq.end()
            }
            Error::Rule { errors } => {
                let mut seq = serializer.serialize_seq(Some(1))?;
                for error in errors {
                    seq.serialize_element(&serde_json::json!({
                        "message": error.message,
                        "locations": error.locations.iter().map(|pos| serde_json::json!({"line": pos.line, "column": pos.column})).collect_vec(),
                    }))?;
                }
                seq.end()
            }
        }
    }
}

struct GQLErrorPos<'a>(&'a Pos);

impl<'a> Serialize for GQLErrorPos<'a> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("line", &self.0.line)?;
        map.serialize_entry("column", &self.0.column)?;
        map.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pos;
    use serde_json::json;

    #[test]
    fn test_request() {
        let request: GQLRequestPart = serde_json::from_value(json! ({
            "query": "{ a b c }"
        }))
        .unwrap();
        assert!(request.variables.is_none());
        assert!(request.operation_name.is_none());
        assert_eq!(request.query, "{ a b c }");
    }

    #[test]
    fn test_batch_request() {
        let request: BatchGQLRequest = serde_json::from_value(json! ([{
            "query": "{ a b c }"
        }, {
            "query": "{ d e f }"
        }]))
        .unwrap();
        if let BatchGQLRequest::Batch(requests) = request {
            assert_eq!(requests[0].query, "{ a b c }");
            assert_eq!(requests[1].query, "{ d e f }");
        } else {
            panic!("Batch query not parsed as a batch")
        }
    }

    #[test]
    fn test_batch_request_single_operation() {
        let request: BatchGQLRequest = serde_json::from_value(json! ({
            "query": "{ a b c }"
        }))
        .unwrap();
        if let BatchGQLRequest::Single(request) = request {
            assert_eq!(request.query, "{ a b c }");
        } else {
            panic!("Batch query not parsed as a batch")
        }
    }

    #[test]
    fn test_request_with_operation_name() {
        let request: GQLRequestPart = serde_json::from_value(json! ({
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
        let request: GQLRequestPart = serde_json::from_value(json! ({
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

    #[test]
    fn test_response_data() {
        let resp = GQLResponse(Ok(QueryResponse {
            data: json!({"ok": true}),
            extensions: None,
            cache_control: Default::default(),
        }));
        assert_eq!(
            serde_json::to_value(resp).unwrap(),
            json! ({
                "data": {
                    "ok": true,
                }
            })
        );
    }

    #[test]
    fn test_batch_response_data() {
        let gql_resp = GQLResponse(Ok(QueryResponse {
            data: json!({"ok": true}),
            extensions: None,
            cache_control: Default::default(),
        }));
        let resp = BatchGQLResponse::Batch(vec![gql_resp]);
        assert_eq!(
            serde_json::to_value(resp).unwrap(),
            json! ([{
                "data": {
                    "ok": true,
                }
            }])
        );
    }

    #[test]
    fn test_batch_response_mixed() {
        let gql_resp = GQLResponse(Ok(QueryResponse {
            data: json!({"ok": true}),
            extensions: None,
            cache_control: Default::default(),
        }));
        let err = Error::Query {
            pos: Pos {
                line: 10,
                column: 20,
            },
            path: None,
            err: QueryError::FieldError {
                err: "MyErrorMessage".to_owned(),
                extended_error: Some(json!({
                    "code": "MY_TEST_CODE"
                })),
            },
        };
        let resp = BatchGQLResponse::Batch(vec![gql_resp, GQLResponse(Err(err))]);
        assert_eq!(
            serde_json::to_value(resp).unwrap(),
            json! ([{
                "data": {
                    "ok": true,
                }
            }, {
                "errors": [{
                    "message":"MyErrorMessage",
                    "extensions": {
                        "code": "MY_TEST_CODE"
                    },
                    "locations": [{"line": 10, "column": 20}]
                }]
            }])
        );
    }

    #[test]
    fn test_batch_response_single_data() {
        let gql_resp = GQLResponse(Ok(QueryResponse {
            data: json!({"ok": true}),
            extensions: None,
            cache_control: Default::default(),
        }));
        let resp = BatchGQLResponse::Single(gql_resp);
        assert_eq!(
            serde_json::to_value(resp).unwrap(),
            json! ({
                "data": {
                    "ok": true,
                }
            })
        );
    }

    #[test]
    fn test_field_error_with_extension() {
        let err = Error::Query {
            pos: Pos {
                line: 10,
                column: 20,
            },
            path: None,
            err: QueryError::FieldError {
                err: "MyErrorMessage".to_owned(),
                extended_error: Some(json!({
                    "code": "MY_TEST_CODE"
                })),
            },
        };

        let resp = GQLResponse(Err(err.into()));

        assert_eq!(
            serde_json::to_value(resp).unwrap(),
            json!({
                "errors": [{
                    "message":"MyErrorMessage",
                    "extensions": {
                        "code": "MY_TEST_CODE"
                    },
                    "locations": [{"line": 10, "column": 20}]
                }]
            })
        );
    }

    #[test]
    fn test_response_error_with_pos() {
        let resp = GQLResponse(Err(Error::Query {
            pos: Pos {
                line: 10,
                column: 20,
            },
            path: None,
            err: QueryError::NotSupported,
        }));
        assert_eq!(
            serde_json::to_value(resp).unwrap(),
            json!({
                "errors": [{
                    "message":"Not supported.",
                    "locations": [
                        {"line": 10, "column": 20}
                    ]
                }]
            })
        );
    }
}
