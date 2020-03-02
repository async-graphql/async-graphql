use crate::{QueryError, Result, Scalar, Value};
use chrono::{DateTime, TimeZone, Utc};
use std::borrow::Cow;

impl Scalar for DateTime<Utc> {
    fn type_name() -> &'static str {
        "DateTime"
    }

    fn parse(value: Value) -> Result<Self> {
        match value {
            Value::String(s) => Ok(Utc.datetime_from_str(&s, "%+")?),
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
            serde_json::Value::String(s) => Ok(Utc.datetime_from_str(&s, "%+")?),
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
        Ok(self.to_rfc3339().into())
    }
}
