use crate::{InputValueError, InputValueResult, Result, ScalarType, Value};
use async_graphql_derive::Scalar;
use url::Url;

#[Scalar(internal)]
impl ScalarType for Url {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => Ok(Url::parse(&s)?),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        Ok(self.to_string().into())
    }
}
