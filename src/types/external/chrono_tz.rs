use crate::{InputValueError, InputValueResult, Result, ScalarType, Value};
use async_graphql_derive::Scalar;
use chrono_tz::Tz;
use std::str::FromStr;

#[Scalar(internal, name = "TimeZone")]
impl ScalarType for Tz {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => Ok(Tz::from_str(&s)?),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(Tz::name(self))
    }
}
