use crate::{QueryError, Result, Scalar, Value};
use anyhow::Error;
use chrono::{DateTime, TimeZone, Utc};

impl Scalar for DateTime<Utc> {
    fn type_name() -> &'static str {
        "DateTime"
    }

    fn parse(value: Value) -> Result<Self> {
        match value {
            Value::String(s) => Ok(Utc.datetime_from_str(&s, "%+")?),
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
            serde_json::Value::String(s) => Ok(Utc.datetime_from_str(&s, "%+")?),
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
        Ok(self.to_rfc3339().into())
    }
}
