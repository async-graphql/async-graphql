use url::Url;

use crate::{InputValueError, InputValueResult, Scalar, ScalarType, Value};

#[Scalar(internal)]
impl ScalarType for Url {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => Ok(Url::parse(&s)?),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.to_string())
    }
}
