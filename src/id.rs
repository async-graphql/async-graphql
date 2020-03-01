use crate::{QueryError, Result, Scalar, Value};
use std::ops::{Deref, DerefMut};

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

impl Scalar for ID {
    fn type_name() -> &'static str {
        "ID"
    }

    fn parse(value: Value) -> Result<Self> {
        match value {
            Value::Int(n) => Ok(ID(n.as_i64().unwrap().to_string())),
            Value::String(s) => Ok(ID(s)),
            _ => {
                return Err(QueryError::ExpectedType {
                    expect: Self::type_name().to_string(),
                    actual: value,
                }
                .into())
            }
        }
    }

    fn parse_from_json(value: serde_json::Value) -> Result<Self> {
        match value {
            serde_json::Value::Number(n) if n.is_i64() => Ok(ID(n.as_i64().unwrap().to_string())),
            serde_json::Value::String(s) => Ok(ID(s)),
            _ => {
                return Err(QueryError::ExpectedJsonType {
                    expect: Self::type_name().to_string(),
                    actual: value,
                }
                .into())
            }
        }
    }

    fn into_json(self) -> Result<serde_json::Value> {
        Ok(self.0.clone().into())
    }
}
