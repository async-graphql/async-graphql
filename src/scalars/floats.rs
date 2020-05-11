use crate::{GqlInputValueResult, GqlResult, GqlValue, InputValueError, ScalarType};
use async_graphql_derive::GqlScalar;

macro_rules! impl_float_scalars {
    ($($ty:ty),*) => {
        $(
        #[GqlScalar(internal)]
        impl ScalarType for $ty {
            fn type_name() -> &'static str {
                "Float"
            }

            fn description() -> Option<&'static str> {
                Some("The `Float` scalar type represents signed double-precision fractional values as specified by [IEEE 754](https://en.wikipedia.org/wiki/IEEE_floating_point).")
            }

            fn parse(value: GqlValue) -> GqlInputValueResult<Self> {
                match value {
                    GqlValue::Int(n) => Ok(n as Self),
                    GqlValue::Float(n) => Ok(n as Self),
                    _ => Err(InputValueError::ExpectedType(value))
                }
            }

            fn is_valid(value: &GqlValue) -> bool {
                match value {
                    GqlValue::Int(_) | GqlValue::Float(_) => true,
                    _ => false
                }
            }

            fn to_json(&self) -> GqlResult<serde_json::Value> {
                Ok((*self).into())
            }
        }
        )*
    };
}

impl_float_scalars!(f32, f64);
