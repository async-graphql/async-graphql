use crate::{InputValueError, InputValueResult, ScalarType, Value};
use async_graphql_derive::Scalar;
#[cfg(feature = "bson")]
use bson::oid::{self, ObjectId};
use std::convert::TryFrom;
use std::num::ParseIntError;
use std::ops::{Deref, DerefMut};

/// ID scalar
///
/// The input is a `&str`, `String`, `usize` or `uuid::UUID`, and the output is a string.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct ID(String);

impl Deref for ID {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ID {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for ID
where
    T: std::fmt::Display,
{
    fn from(value: T) -> Self {
        ID(value.to_string())
    }
}

impl Into<String> for ID {
    fn into(self) -> String {
        self.0
    }
}

impl TryFrom<ID> for usize {
    type Error = ParseIntError;

    fn try_from(id: ID) -> std::result::Result<Self, Self::Error> {
        id.0.parse()
    }
}

impl TryFrom<ID> for uuid::Uuid {
    type Error = uuid::Error;

    fn try_from(id: ID) -> std::result::Result<Self, Self::Error> {
        uuid::Uuid::parse_str(&id.0)
    }
}

#[cfg(feature = "bson")]
impl TryFrom<ID> for ObjectId {
    type Error = oid::Error;

    fn try_from(id: ID) -> std::result::Result<Self, oid::Error> {
        ObjectId::with_string(&id.0)
    }
}

impl PartialEq<&str> for ID {
    fn eq(&self, other: &&str) -> bool {
        self.0.as_str() == *other
    }
}

#[Scalar(internal)]
impl ScalarType for ID {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Int(n) => Ok(ID(n.to_string())),
            Value::String(s) => Ok(ID(s)),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        match value {
            Value::Int(_) | Value::String(_) => true,
            _ => false,
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.clone())
    }
}
