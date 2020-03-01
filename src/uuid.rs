use crate::{QueryError, Result, Scalar, Value};
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
                    expect: Self::type_name().to_string(),
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
                    expect: Self::type_name().to_string(),
                    actual: value,
                }
                .into())
            }
        }
    }

    fn into_json(self) -> Result<serde_json::Value> {
        Ok(self.to_string().into())
    }
}
