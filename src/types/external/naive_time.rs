use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use crate::{InputValueError, InputValueResult, Scalar, ScalarType, Value};

#[Scalar(internal)]
/// ISO 8601 calendar date without timezone.
/// Format: %Y-%m-%d
///
/// # Examples
///
/// * `1994-11-13`
/// * `2000-02-24`
impl ScalarType for NaiveDate {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => Ok(NaiveDate::parse_from_str(&s, "%Y-%m-%d")?),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.format("%Y-%m-%d").to_string())
    }
}

#[Scalar(internal)]
/// ISO 8601 time without timezone.
/// Allows for the nanosecond precision and optional leap second representation.
/// Format: %H:%M:%S%.f
///
/// # Examples
///
/// * `08:59:60.123`
impl ScalarType for NaiveTime {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => Ok(NaiveTime::parse_from_str(&s, "%H:%M:%S%.f")?),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.format("%H:%M:%S%.f").to_string())
    }
}

#[Scalar(internal)]
/// ISO 8601 combined date and time without timezone.
///
/// # Examples
///
/// * `2015-07-01T08:59:60.123`,
impl ScalarType for NaiveDateTime {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => Ok(NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f")?),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.format("%Y-%m-%dT%H:%M:%S%.f").to_string())
    }
}
