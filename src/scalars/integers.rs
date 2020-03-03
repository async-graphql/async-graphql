use crate::{GQLType, QueryError, Result, Scalar, Value};
use std::borrow::Cow;

macro_rules! impl_integer_scalars {
    ($($ty:ty),*) => {
        $(
        impl Scalar for $ty {
            fn type_name() -> &'static str {
                "Int!"
            }

            fn parse(value: Value) -> Result<Self> {
                match value {
                    Value::Int(n) => Ok(n.as_i64().unwrap() as Self),
                    _ => {
                        return Err(QueryError::ExpectedType {
                            expect: Cow::Borrowed(<Self as Scalar>::type_name()),
                            actual: value,
                        }
                        .into())
                    }
                }
            }

            fn parse_from_json(value: serde_json::Value) -> Result<Self> {
                match value {
                    serde_json::Value::Number(n) if n.is_i64() => Ok(n.as_i64().unwrap() as Self),
                    serde_json::Value::Number(n) => Ok(n.as_f64().unwrap() as Self),
                    _ => {
                        return Err(QueryError::ExpectedJsonType {
                            expect: <Self as GQLType>::type_name(),
                            actual: value,
                        }
                        .into())
                    }
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
