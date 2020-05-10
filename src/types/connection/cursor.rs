use crate::{InputValueError, InputValueResult, Result, ScalarType, Value};
use async_graphql_derive::Scalar;
use std::ops::{Deref, DerefMut};

/// Cursor scalar
///
/// A custom scalar that serializes as a string.
/// https://relay.dev/graphql/connections.htm#sec-Cursor
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Cursor(String);

impl Deref for Cursor {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Cursor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for Cursor
where
    T: std::fmt::Display
{
    fn from(value: T) -> Self {
        Cursor(value.to_string())
    }
}

#[Scalar(internal)]
impl ScalarType for Cursor {
    fn type_name() -> &'static str {
        "Cursor"
    }

    fn parse(value: &Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => Ok(Cursor(s.into())),
            _ => Err(InputValueError::ExpectedType),
        }
    }

    fn is_valid(value: &Value) -> bool {
        match value {
            Value::String(_) => true,
            _ => false,
        }
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        Ok(self.0.to_string().into())
    }
}
