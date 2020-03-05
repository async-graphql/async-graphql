mod graphiql_source;
mod playground_source;

pub use graphiql_source::graphiql_source;
pub use playground_source::playground_source;

use crate::{GQLObject, PositionError, Result, Schema, Variables};
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use std::ops::Deref;

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub struct GQLRequest {
    pub query: String,
    #[serde(rename = "operationName")]
    pub operation_name: Option<String>,
    pub variables: Option<serde_json::Value>,
}

impl GQLRequest {
    pub async fn execute<Query, Mutation>(self, schema: &Schema<Query, Mutation>) -> GQLResponse
    where
        Query: GQLObject + Send + Sync,
        Mutation: GQLObject + Send + Sync,
    {
        let vars = match self.variables {
            Some(value) => match Variables::parse_from_json(value) {
                Ok(vars) => Some(vars),
                Err(err) => return GQLResponse(Err(err)),
            },
            None => None,
        };
        let query = schema.query(&self.query);
        let query = match &vars {
            Some(vars) => query.variables(vars),
            None => query,
        };
        let query = match &self.operation_name {
            Some(operation_name) => query.operator_name(operation_name),
            None => query,
        };
        GQLResponse(query.execute().await)
    }
}

pub struct GQLResponse(Result<serde_json::Value>);

impl Serialize for GQLResponse {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match &self.0 {
            Ok(res) => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_key("data")?;
                map.serialize_value(&res)?;
                map.end()
            }
            Err(err) => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_key("errors")?;
                map.serialize_value(&[GQLError(err)])?;
                map.end()
            }
        }
    }
}

struct GQLError<'a>(&'a anyhow::Error);

impl<'a> Deref for GQLError<'a> {
    type Target = anyhow::Error;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> Serialize for GQLError<'a> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        match self.0.downcast_ref::<PositionError>() {
            Some(err) => {
                map.serialize_key("message")?;
                map.serialize_value(&err.to_string())?;

                map.serialize_key("locations")?;
                map.serialize_value(&[serde_json::json! ({
                    "line": err.position.line,
                    "column": err.position.column,
                })])?;
            }
            None => {
                map.serialize_key("message")?;
                map.serialize_value(&self.0.to_string())?;
            }
        }

        map.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ErrorWithPosition;
    use graphql_parser::Pos;
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

    #[test]
    fn test_response_data() {
        let resp = GQLResponse(Ok(json!({"ok": true})));
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
    fn test_response_error() {
        let resp = GQLResponse(Err(anyhow::anyhow!("error")));
        assert_eq!(
            serde_json::to_value(resp).unwrap(),
            json!({
                "errors": [{
                    "message":"error"
                }]
            })
        );
    }

    #[test]
    fn test_response_error_with_pos() {
        let resp = GQLResponse(Err(anyhow::anyhow!("error")
            .with_position(Pos {
                line: 10,
                column: 20,
            })
            .into()));
        assert_eq!(
            serde_json::to_value(resp).unwrap(),
            json!({
                "errors": [{
                    "message":"error",
                    "locations": [
                        {"line": 10, "column": 20}
                    ]
                }]
            })
        );
    }
}
