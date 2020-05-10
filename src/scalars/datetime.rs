use crate::{InputValueError, InputValueResult, Result, ScalarType, Value};
use async_graphql_derive::Scalar;
use chrono::{DateTime, TimeZone, Utc};

/// Implement the DateTime<Utc> scalar
///
/// The input/output is a string in RFC3339 format.
#[Scalar(internal)]
impl ScalarType for DateTime<Utc> {
    fn type_name() -> &'static str {
        "DateTime"
    }

    fn parse(value: &Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => Ok(Utc.datetime_from_str(&s, "%+")?),
            _ => Err(InputValueError::ExpectedType),
        }
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        Ok(self.to_rfc3339().into())
    }
}
