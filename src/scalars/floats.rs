use crate::{InputValueError, InputValueResult, ScalarType, Value};
use async_graphql_derive::Scalar;

/// The `Float` scalar type represents signed double-precision fractional values as specified by [IEEE 754](https://en.wikipedia.org/wiki/IEEE_floating_point).
#[Scalar(internal, name = "Float")]
impl ScalarType for f32 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Number(n) => Ok(n
                .as_f64()
                .ok_or_else(|| InputValueError::from("Invalid number"))?
                as Self),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        match value {
            Value::Number(_) => true,
            _ => false,
        }
    }

    fn to_value(&self) -> Value {
        Value::Number(serde_json::Number::from_f64(*self as f64).unwrap())
    }
}

/// The `Float` scalar type represents signed double-precision fractional values as specified by [IEEE 754](https://en.wikipedia.org/wiki/IEEE_floating_point).
#[Scalar(internal, name = "Float")]
impl ScalarType for f64 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Number(n) => Ok(n
                .as_f64()
                .ok_or_else(|| InputValueError::from("Invalid number"))?
                as Self),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        match value {
            Value::Number(_) => true,
            _ => false,
        }
    }

    fn to_value(&self) -> Value {
        Value::Number(serde_json::Number::from_f64(*self as f64).unwrap())
    }
}
