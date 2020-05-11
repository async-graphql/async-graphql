use crate::{GqlInputValueResult, GqlResult, GqlValue, InputValueError, ScalarType};
use async_graphql_derive::GqlScalar;
use bson::oid::{self, ObjectId};
use std::convert::TryFrom;
use std::num::ParseIntError;
use std::ops::{Deref, DerefMut};
use uuid::Uuid;

/// ID scalar
///
/// The input is a `&str`, `String`, `usize` or `uuid::UUID`, and the output is a string.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct GqlID(String);

impl Deref for GqlID {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for GqlID {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for GqlID
where
    T: std::fmt::Display,
{
    fn from(value: T) -> Self {
        GqlID(value.to_string())
    }
}

impl Into<String> for GqlID {
    fn into(self) -> String {
        self.0
    }
}

impl TryFrom<GqlID> for usize {
    type Error = ParseIntError;

    fn try_from(id: GqlID) -> std::result::Result<Self, Self::Error> {
        id.0.parse()
    }
}

impl TryFrom<GqlID> for Uuid {
    type Error = uuid::Error;

    fn try_from(id: GqlID) -> std::result::Result<Self, Self::Error> {
        Uuid::parse_str(&id.0)
    }
}

impl TryFrom<GqlID> for ObjectId {
    type Error = oid::Error;

    fn try_from(id: GqlID) -> std::result::Result<Self, oid::Error> {
        ObjectId::with_string(&id.0)
    }
}

impl PartialEq<&str> for GqlID {
    fn eq(&self, other: &&str) -> bool {
        self.0.as_str() == *other
    }
}

#[GqlScalar(internal)]
impl ScalarType for GqlID {
    fn type_name() -> &'static str {
        "ID"
    }

    fn parse(value: GqlValue) -> GqlInputValueResult<Self> {
        match value {
            GqlValue::Int(n) => Ok(GqlID(n.to_string())),
            GqlValue::String(s) => Ok(GqlID(s)),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn is_valid(value: &GqlValue) -> bool {
        match value {
            GqlValue::Int(_) | GqlValue::String(_) => true,
            _ => false,
        }
    }

    fn to_json(&self) -> GqlResult<serde_json::Value> {
        Ok(self.0.clone().into())
    }
}
