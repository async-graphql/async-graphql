use crate::r#type::{GQLInputValue, GQLOutputValue, GQLType};
use crate::{ContextSelectionSet, QueryError, Result};
use graphql_parser::query::Value;

pub trait Scalar: Sized + Send {
    fn type_name() -> &'static str;
    fn parse(value: Value) -> Result<Self>;
    fn parse_from_json(value: serde_json::Value) -> Result<Self>;
    fn into_json(self) -> Result<serde_json::Value>;
}

impl<T: Scalar> GQLType for T {
    fn type_name() -> String {
        T::type_name().to_string()
    }
}

impl<T: Scalar> GQLInputValue for T {
    fn parse(value: Value) -> Result<Self> {
        T::parse(value)
    }

    fn parse_from_json(value: serde_json::Value) -> Result<Self> {
        T::parse_from_json(value)
    }
}

#[async_trait::async_trait]
impl<T: Scalar> GQLOutputValue for T {
    async fn resolve(self, _: &ContextSelectionSet<'_>) -> Result<serde_json::Value> {
        T::into_json(self)
    }
}

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
                            expect: <Self as Scalar>::type_name().to_string(),
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
                            expect: <Self as GQLType>::type_name().to_string(),
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
        impl Scalar for $ty {
            fn type_name() -> &'static str {
                "Float!"
            }

            fn parse(value: Value) -> Result<Self> {
                match value {
                    Value::Int(n) => Ok(n.as_i64().unwrap() as Self),
                    Value::Float(n) => Ok(n as Self),
                    _ => {
                        return Err(QueryError::ExpectedType {
                            expect: <Self as Scalar>::type_name().to_string(),
                            actual: value,
                        }
                        .into())
                    }
                }
            }

            fn parse_from_json(value: serde_json::Value) -> Result<Self> {
                match value {
                    serde_json::Value::Number(n) => Ok(n.as_f64().unwrap() as Self),
                    _ => {
                        return Err(QueryError::ExpectedJsonType {
                            expect: <Self as GQLType>::type_name().to_string(),
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

impl Scalar for String {
    fn type_name() -> &'static str {
        "String!"
    }

    fn parse(value: Value) -> Result<Self> {
        match value {
            Value::String(s) => Ok(s),
            _ => {
                return Err(QueryError::ExpectedType {
                    expect: <Self as GQLType>::type_name().to_string(),
                    actual: value,
                }
                .into())
            }
        }
    }

    fn parse_from_json(value: serde_json::Value) -> Result<Self> {
        match value {
            serde_json::Value::String(s) => Ok(s),
            _ => {
                return Err(QueryError::ExpectedJsonType {
                    expect: <Self as GQLType>::type_name().to_string(),
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

impl Scalar for bool {
    fn type_name() -> &'static str {
        "Boolean!"
    }

    fn parse(value: Value) -> Result<Self> {
        match value {
            Value::Boolean(n) => Ok(n),
            _ => {
                return Err(QueryError::ExpectedType {
                    expect: <Self as GQLType>::type_name().to_string(),
                    actual: value,
                }
                .into())
            }
        }
    }

    fn parse_from_json(value: serde_json::Value) -> Result<Self> {
        match value {
            serde_json::Value::Bool(n) => Ok(n),
            _ => {
                return Err(QueryError::ExpectedJsonType {
                    expect: <Self as GQLType>::type_name().to_string(),
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
