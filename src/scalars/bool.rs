use crate::{InputValueError, InputValueResult, Result, ScalarType, Value};
use async_graphql_derive::Scalar;

/// The `Boolean` scalar type represents `true` or `false`.
#[Scalar(internal, name = "Boolean")]
impl ScalarType for bool {
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
