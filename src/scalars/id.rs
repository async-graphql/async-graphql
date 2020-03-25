use crate::{impl_scalar_internal, Result, Scalar, Value};
use std::ops::{Deref, DerefMut};

/// ID scalar
///
/// The input is a string or integer, and the output is a string.
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

impl Scalar for ID {
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

impl_scalar_internal!(ID);
