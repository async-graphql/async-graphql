use crate::{InputValueError, InputValueResult, Result, ScalarType, Value};
use async_graphql_derive::Scalar;

#[Scalar(internal)]
impl ScalarType for bool {
    fn type_name() -> &'static str {
        "Boolean"
    }

    fn description() -> Option<&'static str> {
        Some("The `Boolean` scalar type represents `true` or `false`.")
    }

    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Boolean(n) => Ok(n),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        match value {
            Value::Boolean(_) => true,
            _ => false,
        }
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        Ok((*self).into())
    }
}
