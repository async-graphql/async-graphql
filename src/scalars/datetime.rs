use crate::{InputValueError, InputValueResult, ScalarType, Value};
use async_graphql_derive::Scalar;
use chrono::{DateTime, FixedOffset, Utc};

/// Implement the DateTime<FixedOffset> scalar
///
/// The input/output is a string in RFC3339 format.
#[Scalar(internal, name = "DateTimeFixedOffset")]
impl ScalarType for DateTime<FixedOffset> {
    fn parse(value: Value) -> InputValueResult<Self> {
        match &value {
            Value::String(s) => Ok(DateTime::parse_from_rfc3339(s)?),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.to_rfc3339())
    }
}

/// Implement the DateTime<Utc> scalar
///
/// The input/output is a string in RFC3339 format.
#[Scalar(internal, name = "DateTimeUtc")]
impl ScalarType for DateTime<Utc> {
    fn parse(value: Value) -> InputValueResult<Self> {
        match &value {
            Value::String(s) => Ok(s.parse::<DateTime<Utc>>()?),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.to_rfc3339())
    }
}
