use crate::{Result, ScalarType, Value};
use async_graphql_derive::Scalar;
use std::convert::TryInto;
use std::num::ParseIntError;
use std::ops::{Deref, DerefMut};
use uuid::Uuid;

/// ID scalar
///
/// The input is a `&str`, `String`, `usize` or `uuid::UUID`, and the output is a string.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct ID(String);

impl std::fmt::Display for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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

impl From<String> for ID {
    fn from(value: String) -> Self {
        ID(value)
    }
}

impl Into<String> for ID {
    fn into(self) -> String {
        self.0
    }
}

impl<'a> From<&'a str> for ID {
    fn from(value: &'a str) -> Self {
        ID(value.to_string())
    }
}

impl From<usize> for ID {
    fn from(value: usize) -> Self {
        ID(value.to_string())
    }
}

impl TryInto<usize> for ID {
    type Error = ParseIntError;

    fn try_into(self) -> std::result::Result<usize, Self::Error> {
        self.0.parse()
    }
}

impl From<Uuid> for ID {
    fn from(uuid: Uuid) -> ID {
        ID(uuid.to_string())
    }
}

impl TryInto<Uuid> for ID {
    type Error = uuid::Error;

    fn try_into(self) -> std::result::Result<Uuid, Self::Error> {
        Uuid::parse_str(&self.0)
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

    fn parse(value: &Value) -> Option<Self> {
        match value {
            Value::Int(n) => Some(ID(n.as_i64().unwrap().to_string())),
            Value::String(s) => Some(ID(s.clone())),
            _ => None,
        }
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        Ok(self.0.clone().into())
    }
}
