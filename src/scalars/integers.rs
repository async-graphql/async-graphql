use crate::{impl_scalar_internal, Result, Scalar, Value};

macro_rules! impl_integer_scalars {
    ($($ty:ty),*) => {
        $(
        impl Scalar for $ty {
            fn type_name() -> &'static str {
                "Int"
            }

            fn description() -> Option<&'static str> {
                Some("The `Int` scalar type represents non-fractional signed whole numeric values. Int can represent values between -(2^31) and 2^31 - 1.")
            }

            fn parse(value: &Value) -> Option<Self> {
                match value {
                    Value::Int(n) => Some(n.as_i64().unwrap() as Self),
                    _ => None
                }
            }

            fn to_json(&self) -> Result<serde_json::Value> {
                Ok((*self).into())
            }
        }

        impl_scalar_internal!($ty);
        )*
    };
}

impl_integer_scalars!(i8, i16, i32, i64, u8, u16, u32, u64);
