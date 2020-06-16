use crate::{InputValueError, InputValueResult, ScalarType, Value};
use async_graphql_derive::Scalar;
use chrono::{DateTime, Utc};

/// Implement the DateTime<Utc> scalar
///
/// The input/output is a string in RFC3339 format.
#[Scalar(internal, name = "DateTimeUtc")]
impl ScalarType for DateTime<Utc> {
    fn parse(value: Value) -> InputValueResult<Self> {
        match &value {
            Value::String(s) => s
                .parse::<DateTime<Utc>>()
                .map_err(|_| InputValueError::ExpectedType(value)),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.to_rfc3339())
    }
}
