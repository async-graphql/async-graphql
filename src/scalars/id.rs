use crate::{InputValueError, InputValueResult, Result, ScalarType, Value};
use async_graphql_derive::Scalar;
use bson::oid::{self, ObjectId};
use std::convert::TryInto;
use std::num::ParseIntError;
use std::ops::{Deref, DerefMut};
use uuid::Uuid;

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

impl TryInto<usize> for ID {
    type Error = ParseIntError;

    fn try_into(self) -> std::result::Result<usize, Self::Error> {
        self.0.parse()
    }
}

impl TryInto<Uuid> for ID {
    type Error = uuid::Error;

    fn try_into(self) -> std::result::Result<Uuid, Self::Error> {
        Uuid::parse_str(&self.0)
    }
}

impl TryInto<ObjectId> for ID {
    type Error = oid::Error;

    fn try_into(self) -> std::result::Result<ObjectId, oid::Error> {
        ObjectId::with_string(&self.0)
    }
}

impl PartialEq<&str> for ID {
    fn eq(&self, other: &&str) -> bool {
        self.0.as_str() == *other
    }
}

#[Scalar(internal)]
impl ScalarType for ID {
    fn type_name() -> &'static str {
        "ID"
    }

    fn parse(value: &Value) -> InputValueResult<Self> {
        match value {
            Value::Int(n) => Ok(ID(n.to_string())),
            Value::String(s) => Ok(ID(s.clone())),
            _ => Err(InputValueError::ExpectedType),
        }
    }

    fn is_valid(value: &Value) -> bool {
        match value {
            Value::Int(_) | Value::String(_) => true,
            _ => false,
        }
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        Ok(self.0.clone().into())
    }
}
