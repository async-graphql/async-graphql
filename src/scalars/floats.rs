use crate::{impl_scalar, GQLScalar, Result, Value};

macro_rules! impl_float_scalars {
    ($($ty:ty),*) => {
        $(
        impl GQLScalar for $ty {
            fn type_name() -> &'static str {
                "Float"
            }

            fn description() -> Option<&'static str> {
                Some("The `Float` scalar type represents signed double-precision fractional values as specified by [IEEE 754](https://en.wikipedia.org/wiki/IEEE_floating_point).")
            }

            fn parse(value: &Value) -> Option<Self> {
                match value {
                    Value::Int(n) => Some(n.as_i64().unwrap() as Self),
                    Value::Float(n) => Some(*n as Self),
                    _ => None
                }
            }

            fn to_json(&self) -> Result<serde_json::Value> {
                Ok((*self).into())
            }
        }

        impl_scalar!($ty);
        )*
    };
}

impl_float_scalars!(f32, f64);
