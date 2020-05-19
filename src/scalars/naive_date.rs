use crate::{InputValueError, InputValueResult, Result, ScalarType, Value};
use async_graphql_derive::Scalar;
use chrono::NaiveDate;

#[Scalar(internal)]
impl ScalarType for NaiveDate {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => Ok(NaiveDate::parse_from_str(&s, "%Y-%m-%d")?),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        Ok(self.format("%Y-%m-%d").to_string().into())
    }
}
