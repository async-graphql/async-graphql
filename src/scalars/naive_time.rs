use crate::{InputValueError, InputValueResult, ScalarType, Value};
use async_graphql_derive::Scalar;
use chrono::NaiveTime;

#[Scalar(internal)]
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
