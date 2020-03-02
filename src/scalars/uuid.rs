use crate::{QueryError, Result, Scalar, Value};
use std::borrow::Cow;
use uuid::Uuid;

impl Scalar for Uuid {
    fn type_name() -> &'static str {
        "UUID"
    }

    fn parse(value: Value) -> Result<Self> {
        match value {
            Value::String(s) => Ok(Uuid::parse_str(&s)?),
            _ => {
                return Err(QueryError::ExpectedType {
                    expect: Cow::Borrowed(Self::type_name()),
                    actual: value,
                }
                .into())
            }
        }
    }

    fn parse_from_json(value: serde_json::Value) -> Result<Self> {
        match value {
            serde_json::Value::String(s) => Ok(Uuid::parse_str(&s)?),
            _ => {
                return Err(QueryError::ExpectedJsonType {
                    expect: Cow::Borrowed(Self::type_name()),
                    actual: value,
                }
                .into())
            }
        }
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        Ok(self.to_string().into())
    }
}
