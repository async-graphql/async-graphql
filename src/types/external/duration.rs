use chrono::Duration;
use iso8601_duration as iso8601;

use crate::{InputValueError, InputValueResult, Scalar, ScalarType, Value};

/// Implement the Duration scalar
///
/// The input/output is a string in ISO8601 format.
#[Scalar(internal, name = "Duration", specified_by_url = "https://en.wikipedia.org/wiki/ISO_8601#Durations")]
impl ScalarType for Duration {
    fn parse(value: Value) -> InputValueResult<Self> {
        match &value {
            Value::String(s) => Ok(Duration::from_std(iso8601::Duration::parse(s)?.to_std())?),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.to_string())
    }
}
