use crate::{Result, Scalar, Value};

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

            fn parse_from_json(value: &serde_json::Value) -> Option<Self> {
                match value {
                    serde_json::Value::Number(n) if n.is_i64() => Some(n.as_i64().unwrap() as Self),
                    serde_json::Value::Number(n) => Some(n.as_f64().unwrap() as Self),
                    _ => None,
                }
            }

            fn to_json(&self) -> Result<serde_json::Value> {
                Ok((*self).into())
            }
        }
        )*
    };
}

impl_integer_scalars!(i8, i16, i32, i64, u8, u16, u32, u64);
