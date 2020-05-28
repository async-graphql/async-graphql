use crate::{InputValueError, InputValueResult, ScalarType, Value};
use async_graphql_derive::Scalar;

/// The `Float` scalar type represents signed double-precision fractional values as specified by [IEEE 754](https://en.wikipedia.org/wiki/IEEE_floating_point).
#[Scalar(internal, name = "Float")]
impl ScalarType for f32 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Int(n) => Ok(n as Self),
            Value::Float(n) => Ok(n as Self),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        match value {
            Value::Int(_) | Value::Float(_) => true,
            _ => false,
        }
    }

    fn to_value(&self) -> Value {
        Value::Float(*self as f64)
    }
}

/// The `Float` scalar type represents signed double-precision fractional values as specified by [IEEE 754](https://en.wikipedia.org/wiki/IEEE_floating_point).
#[Scalar(internal, name = "Float")]
impl ScalarType for f64 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Int(n) => Ok(n as Self),
            Value::Float(n) => Ok(n as Self),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        match value {
            Value::Int(_) | Value::Float(_) => true,
            _ => false,
        }
    }

    fn to_value(&self) -> Value {
        Value::Float(*self as f64)
    }
}
