use crate::{InputValueError, InputValueResult, Result, ScalarType, Value};
use async_graphql_derive::Scalar;
use uuid::Uuid;

#[Scalar(internal, name = "UUID")]
impl ScalarType for Uuid {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => Ok(Uuid::parse_str(&s)?),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        Ok(self.to_string().into())
    }
}
