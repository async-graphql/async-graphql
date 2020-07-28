use crate::{InputValueError, InputValueResult, ScalarType, Value};
use async_graphql_derive::Scalar;

macro_rules! int_scalar {
    ($($ty:ty),*) => {
        $(
        /// The `Int` scalar type represents non-fractional whole numeric values.
        #[Scalar(internal, name = "Int")]
        impl ScalarType for $ty {
            fn parse(value: Value) -> InputValueResult<Self> {
                match value {
                    Value::Number(n) => {
                        let n = n
                            .as_i64()
                            .ok_or_else(|| InputValueError::from("Invalid number"))?;
                        if n < Self::MIN as i64 || n > Self::MAX as i64 {
                            return Err(InputValueError::from(format!(
                                "Only integers from {} to {} are accepted.",
                                Self::MIN,
                                Self::MAX
                            )));
                        }
                        Ok(n as Self)
                    }
                    _ => Err(InputValueError::ExpectedType(value)),
                }
            }

            fn is_valid(value: &Value) -> bool {
                match value {
                    Value::Number(n) if n.is_i64() => true,
                    _ => false,
                }
            }

            fn to_value(&self) -> Value {
                Value::Number(serde_json::Number::from(*self as i64))
            }
        }
        )*
    };
}

macro_rules! uint_scalar {
    ($($ty:ty),*) => {
        $(
        /// The `Int` scalar type represents non-fractional whole numeric values.
        #[Scalar(internal, name = "Int")]
        impl ScalarType for $ty {
            fn parse(value: Value) -> InputValueResult<Self> {
                match value {
                    Value::Number(n) => {
                        let n = n
                            .as_u64()
                            .ok_or_else(|| InputValueError::from("Invalid number"))?;
                        if n > Self::MAX as u64 {
                            return Err(InputValueError::from(format!(
                                "Only integers from {} to {} are accepted.",
                                0,
                                Self::MAX
                            )));
                        }
                        Ok(n as Self)
                    }
                    _ => Err(InputValueError::ExpectedType(value)),
                }
            }

            fn is_valid(value: &Value) -> bool {
                match value {
                    Value::Number(n) if n.is_u64() => true,
                    _ => false,
                }
            }

            fn to_value(&self) -> Value {
                Value::Number(serde_json::Number::from(*self as u64))
            }
        }
        )*
    };
}

int_scalar!(i8, i16, i32, i64);
uint_scalar!(u8, u16, u32, u64);
