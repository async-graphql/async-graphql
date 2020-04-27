use crate::{Result, ScalarType, Value};
use async_graphql_derive::Scalar;

#[Scalar(internal)]
impl ScalarType for bool {
    fn type_name() -> &'static str {
        "Boolean"
    }

    fn description() -> Option<&'static str> {
        Some("The `Boolean` scalar type represents `true` or `false`.")
    }

    fn parse(value: &Value) -> Option<Self> {
        match value {
            Value::Boolean(n) => Some(*n),
            _ => None,
        }
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        Ok((*self).into())
    }
}
