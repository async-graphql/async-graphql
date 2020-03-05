use crate::{Result, GQLScalar, Value};
use chrono::{DateTime, TimeZone, Utc};

impl GQLScalar for DateTime<Utc> {
    fn type_name() -> &'static str {
        "DateTime"
    }

    fn parse(value: &Value) -> Option<Self> {
        match value {
            Value::String(s) => Some(Utc.datetime_from_str(&s, "%+").ok()?),
            _ => None,
        }
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        Ok(self.to_rfc3339().into())
    }
}
