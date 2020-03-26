//! A helper module that supports HTTP

mod graphiql_source;
mod playground_source;

pub use graphiql_source::graphiql_source;
pub use playground_source::playground_source;

use crate::error::{RuleError, RuleErrors};
use crate::query::PreparedQuery;
use crate::{ObjectType, PositionError, QueryResult, Result, Schema, SubscriptionType, Variables};
use graphql_parser::Pos;
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Serialize, Serializer};
use std::ops::Deref;

/// GraphQL Request object
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

impl GQLRequest {
    /// Execute the query and return the `GQLResponse`.
    pub async fn execute<Query, Mutation, Subscription>(
        mut self,
        schema: &Schema<Query, Mutation, Subscription>,
    ) -> GQLResponse
    where
        Query: ObjectType + Send + Sync,
        Mutation: ObjectType + Send + Sync,
        Subscription: SubscriptionType + Send + Sync,
    {
        match self.prepare(schema) {
            Ok(query) => GQLResponse(query.execute().await),
            Err(err) => GQLResponse(Err(err)),
        }
    }

    /// Prepare a query and return a `PreparedQuery` object that gets some information about the query.
    pub fn prepare<'a, Query, Mutation, Subscription>(
        &'a mut self,
        schema: &'a Schema<Query, Mutation, Subscription>,
    ) -> Result<PreparedQuery<'a, Query, Mutation>>
    where
        Query: ObjectType + Send + Sync,
        Mutation: ObjectType + Send + Sync,
        Subscription: SubscriptionType + Send + Sync,
    {
        let vars = match self.variables.take() {
            Some(value) => match Variables::parse_from_json(value) {
                Ok(vars) => Some(vars),
                Err(err) => return Err(err),
            },
            None => None,
        };
        let query = schema.query(&self.query);
        let query = match vars {
            Some(vars) => query.variables(vars),
            None => query,
        };
        let query = match &self.operation_name {
            Some(name) => query.operator_name(name),
            None => query,
        };
        query.prepare()
    }
}

/// Serializable query result type
pub struct GQLResponse(pub Result<QueryResult>);

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

/// Serializable error type
pub struct GQLError<'a>(pub &'a anyhow::Error);

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
        if let Some(err) = self.0.downcast_ref::<PositionError>() {
            let mut seq = serializer.serialize_seq(Some(1))?;
            seq.serialize_element(&GQLPositionError(err))?;
            seq.end()
        } else if let Some(err) = self.0.downcast_ref::<RuleErrors>() {
            let mut seq = serializer.serialize_seq(Some(err.errors.len()))?;
            for err in &err.errors {
                seq.serialize_element(&GQLRuleError(err))?;
            }
            seq.end()
        } else {
            let mut seq = serializer.serialize_seq(None)?;
            seq.serialize_element(&serde_json::json!({
                "message": self.0.to_string(),
            }))?;
            seq.end()
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

struct GQLPositionError<'a>(&'a PositionError);

impl<'a> Serialize for GQLPositionError<'a> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("message", &self.0.inner.to_string())?;
        map.serialize_entry(
            "locations",
            std::slice::from_ref(&GQLErrorPos(&self.0.position)),
        )?;
        map.end()
    }
}

struct GQLRuleError<'a>(&'a RuleError);

impl<'a> Serialize for GQLRuleError<'a> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("message", &self.0.message)?;
        map.serialize_entry(
            "locations",
            &self
                .0
                .locations
                .iter()
                .map(|pos| GQLErrorPos(pos))
                .collect::<Vec<_>>(),
        )?;
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
        let resp = GQLResponse(Ok(QueryResult {
            data: json!({"ok": true}),
            extensions: None,
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
