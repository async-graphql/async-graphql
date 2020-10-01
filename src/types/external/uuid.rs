use crate::{InputValueError, InputValueResult, Scalar, ScalarType, Value};
use uuid::Uuid;

#[Scalar(internal, name = "UUID")]
impl ScalarType for Uuid {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => Ok(Uuid::parse_str(&s)?),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.to_string())
    }
}
