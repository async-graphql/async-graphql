use crate::r#type::{GQLInputValue, GQLOutputValue, GQLType};
use crate::{ContextSelectionSet, GQLQueryError, Result};
use graphql_parser::query::Value;

pub trait GQLScalar: Sized + Send {
    fn type_name() -> &'static str;
    fn parse(value: Value) -> Result<Self>;
    fn into_json(self) -> Result<serde_json::Value>;
}

impl<T: GQLScalar> GQLType for T {
    fn type_name() -> String {
        T::type_name().to_string()
    }
}

impl<T: GQLScalar> GQLInputValue for T {
    fn parse(value: Value) -> Result<Self> {
        T::parse(value)
    }
}

#[async_trait::async_trait]
impl<T: GQLScalar> GQLOutputValue for T {
    async fn resolve(self, _: &ContextSelectionSet<'_>) -> Result<serde_json::Value> {
        T::into_json(self)
    }
}

macro_rules! impl_integer_scalars {
    ($($ty:ty),*) => {
        $(
        impl GQLScalar for $ty {
            fn type_name() -> &'static str {
                "Int!"
            }

            fn parse(value: Value) -> Result<Self> {
                match value {
                    Value::Int(n) => Ok(n.as_i64().unwrap() as Self),
                    _ => {
                        return Err(GQLQueryError::ExpectedType {
                            expect: <Self as GQLScalar>::type_name().to_string(),
                            actual: value,
                        }
                        .into())
                    }
                }
            }

            fn into_json(self) -> Result<serde_json::Value> {
                Ok(self.into())
            }
        }
        )*
    };
}

impl_integer_scalars!(i8, i16, i32, i64, u8, u16, u32, u64);

macro_rules! impl_float_scalars {
    ($($ty:ty),*) => {
        $(
        impl GQLScalar for $ty {
            fn type_name() -> &'static str {
                "Float!"
            }

            fn parse(value: Value) -> Result<Self> {
                match value {
                    Value::Int(n) => Ok(n.as_i64().unwrap() as Self),
                    Value::Float(n) => Ok(n as Self),
                    _ => {
                        return Err(GQLQueryError::ExpectedType {
                            expect: <Self as GQLScalar>::type_name().to_string(),
                            actual: value,
                        }
                        .into())
                    }
                }
            }

            fn into_json(self) -> Result<serde_json::Value> {
                Ok(self.into())
            }
        }
        )*
    };
}

impl_float_scalars!(f32, f64);

impl GQLScalar for String {
    fn type_name() -> &'static str {
        "String!"
    }

    fn parse(value: Value) -> Result<Self> {
        match value {
            Value::String(s) => Ok(s),
            _ => {
                return Err(GQLQueryError::ExpectedType {
                    expect: <Self as GQLScalar>::type_name().to_string(),
                    actual: value,
                }
                .into())
            }
        }
    }

    fn into_json(self) -> Result<serde_json::Value> {
        Ok(self.into())
    }
}

impl GQLScalar for bool {
    fn type_name() -> &'static str {
        "Boolean!"
    }

    fn parse(value: Value) -> Result<Self> {
        match value {
            Value::Boolean(n) => Ok(n),
            _ => {
                return Err(GQLQueryError::ExpectedType {
                    expect: <Self as GQLScalar>::type_name().to_string(),
                    actual: value,
                }
                .into())
            }
        }
    }

    fn into_json(self) -> Result<serde_json::Value> {
        Ok(self.into())
    }
}
