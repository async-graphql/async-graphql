use chrono::{DateTime, FixedOffset, Local, Utc};

use crate::{InputValueError, InputValueResult, Scalar, ScalarType, Value};

/// Implement the DateTime<FixedOffset> scalar
///
/// The input/output is a string in RFC3339 format.
#[Scalar(internal, name = "DateTime")]
impl ScalarType for DateTime<FixedOffset> {
    fn parse(value: Value) -> InputValueResult<Self> {
        match &value {
            Value::String(s) => Ok(s.parse::<DateTime<FixedOffset>>()?),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.to_rfc3339())
    }
}

/// Implement the DateTime<Local> scalar
///
/// The input/output is a string in RFC3339 format.
#[Scalar(internal, name = "DateTime")]
impl ScalarType for DateTime<Local> {
    fn parse(value: Value) -> InputValueResult<Self> {
        match &value {
            Value::String(s) => Ok(s.parse::<DateTime<Local>>()?),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.to_rfc3339())
    }
}

/// Implement the DateTime<Utc> scalar
///
/// The input/output is a string in RFC3339 format.
#[Scalar(internal, name = "DateTime")]
impl ScalarType for DateTime<Utc> {
    fn parse(value: Value) -> InputValueResult<Self> {
        match &value {
            Value::String(s) => Ok(s.parse::<DateTime<Utc>>()?),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.to_rfc3339())
    }
}
