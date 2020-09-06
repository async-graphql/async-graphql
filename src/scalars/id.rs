use crate::{InputValueError, InputValueResult, ScalarType, Value};
use async_graphql_derive::Scalar;
#[cfg(feature = "bson")]
use bson::oid::{self, ObjectId};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::num::ParseIntError;
use std::ops::{Deref, DerefMut};

/// ID scalar
///
/// The input is a `&str`, `String`, `usize` or `uuid::UUID`, and the output is a string.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
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

macro_rules! try_from_integers {
    ($($ty:ty),*) => {
        $(
           impl TryFrom<ID> for $ty {
                type Error = ParseIntError;

                fn try_from(id: ID) -> std::result::Result<Self, Self::Error> {
                    id.0.parse()
                }
            }
         )*
    };
}

try_from_integers!(i8, i16, i32, i64, u8, u16, u32, u64, isize, usize);

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
            Value::Number(n) if n.is_i64() => Ok(ID(n.to_string())),
            Value::String(s) => Ok(ID(s)),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        match value {
            Value::Number(n) if n.is_i64() => true,
            Value::String(_) => true,
            _ => false,
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.clone())
    }
}
