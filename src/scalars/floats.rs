use crate::{InputValueError, InputValueResult, Result, ScalarType, Value};
use async_graphql_derive::Scalar;

macro_rules! impl_float_scalars {
    ($($ty:ty),*) => {
        $(
        /// The `Float` scalar type represents signed double-precision fractional values as specified by [IEEE 754](https://en.wikipedia.org/wiki/IEEE_floating_point).
        #[Scalar(internal, name = "Float")]
        impl ScalarType for $ty {
            fn parse(value: Value) -> InputValueResult<Self> {
                match value {
                    Value::Int(n) => Ok(n as Self),
                    Value::Float(n) => Ok(n as Self),
                    _ => Err(InputValueError::ExpectedType(value))
                }
            }

            fn is_valid(value: &Value) -> bool {
                match value {
                    Value::Int(_) | Value::Float(_) => true,
                    _ => false
                }
            }

            fn to_json(&self) -> Result<serde_json::Value> {
                Ok((*self).into())
            }
        }
        )*
    };
}

impl_float_scalars!(f32, f64);
