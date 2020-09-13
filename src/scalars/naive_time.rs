use crate::{GQLScalar, InputValueError, InputValueResult, ScalarType, Value};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

#[GQLScalar(internal)]
impl ScalarType for NaiveDate {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => Ok(NaiveDate::parse_from_str(&s, "%Y-%m-%d")?),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.format("%Y-%m-%d").to_string())
    }
}

#[GQLScalar(internal)]
impl ScalarType for NaiveTime {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => Ok(NaiveTime::parse_from_str(&s, "%H:%M:%S")?),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.format("%H:%M:%S").to_string())
    }
}

#[GQLScalar(internal)]
impl ScalarType for NaiveDateTime {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => Ok(NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")?),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.format("%Y-%m-%d %H:%M:%S").to_string())
    }
}
