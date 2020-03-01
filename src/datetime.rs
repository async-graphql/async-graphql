use crate::{GQLQueryError, GQLScalar, Result, Value};
use chrono::{DateTime, TimeZone, Utc};

impl GQLScalar for DateTime<Utc> {
    fn type_name() -> &'static str {
        "DateTime"
    }

    fn parse(value: Value) -> Result<Self> {
        match value {
            Value::String(s) => Ok(Utc.datetime_from_str(&s, "%+")?),
            _ => {
                return Err(GQLQueryError::ExpectedType {
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
