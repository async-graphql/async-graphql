use crate::{InputValueError, InputValueResult, ScalarType, Value};
use async_graphql_derive::Scalar;

/// The `Int` scalar type represents non-fractional signed whole numeric values. Int can represent values between -(2^31) and 2^31 - 1.
#[Scalar(internal, name = "Int")]
impl ScalarType for i8 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Int(n) => Ok(n as Self),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        match value {
            Value::Int(_) => true,
            _ => false,
        }
    }

    fn to_value(&self) -> Value {
        Value::Int(*self as i32)
    }
}

/// The `Int` scalar type represents non-fractional signed whole numeric values. Int can represent values between -(2^31) and 2^31 - 1.
#[Scalar(internal, name = "Int")]
impl ScalarType for i16 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Int(n) => Ok(n as Self),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        match value {
            Value::Int(_) => true,
            _ => false,
        }
    }

    fn to_value(&self) -> Value {
        Value::Int(*self as i32)
    }
}

/// The `Int` scalar type represents non-fractional signed whole numeric values. Int can represent values between -(2^31) and 2^31 - 1.
#[Scalar(internal, name = "Int")]
impl ScalarType for i32 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Int(n) => Ok(n as Self),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        match value {
            Value::Int(_) => true,
            _ => false,
        }
    }

    fn to_value(&self) -> Value {
        Value::Int(*self as i32)
    }
}

/// The `Int` scalar type represents non-fractional signed whole numeric values. Int can represent values between -(2^31) and 2^31 - 1.
#[Scalar(internal, name = "Int")]
impl ScalarType for u8 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Int(n) => Ok(n as Self),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        match value {
            Value::Int(_) => true,
            _ => false,
        }
    }

    fn to_value(&self) -> Value {
        Value::Int(*self as i32)
    }
}

/// The `Int` scalar type represents non-fractional signed whole numeric values. Int can represent values between -(2^31) and 2^31 - 1.
#[Scalar(internal, name = "Int")]
impl ScalarType for u16 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Int(n) => Ok(n as Self),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        match value {
            Value::Int(_) => true,
            _ => false,
        }
    }

    fn to_value(&self) -> Value {
        Value::Int(*self as i32)
    }
}

/// The `Int` scalar type represents non-fractional signed whole numeric values. Int can represent values between -(2^31) and 2^31 - 1.
#[Scalar(internal, name = "Int")]
impl ScalarType for u32 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Int(n) => Ok(n as Self),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        match value {
            Value::Int(_) => true,
            _ => false,
        }
    }

    fn to_value(&self) -> Value {
        Value::Int(*self as i32)
    }
}

/// The `Int64` scalar type represents non-fractional signed whole numeric values. Int can represent values between -(2^64) and 2^64 - 1.
#[Scalar(internal, name = "Int64")]
impl ScalarType for i64 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Int(n) => Ok(n as Self),
            Value::String(s) => Ok(s.parse()?),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        match value {
            Value::Int(_) | Value::String(_) => true,
            _ => false,
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.to_string())
    }
}

/// The `Int64` scalar type represents non-fractional signed whole numeric values. Int can represent values between -(2^64) and 2^64 - 1.
#[Scalar(internal, name = "Int64")]
impl ScalarType for u64 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Int(n) => Ok(n as Self),
            Value::String(s) => Ok(s.parse()?),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        match value {
            Value::Int(_) | Value::String(_) => true,
            _ => false,
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.to_string())
    }
}
