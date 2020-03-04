use crate::{Result, Scalar, Value};
use chrono::{DateTime, TimeZone, Utc};

impl Scalar for DateTime<Utc> {
    fn type_name() -> &'static str {
        "DateTime"
    }

    fn parse(value: &Value) -> Option<Self> {
        match value {
            Value::String(s) => Some(Utc.datetime_from_str(&s, "%+").ok()?),
            _ => None,
        }
    }

    fn parse_from_json(value: &serde_json::Value) -> Option<Self> {
        match value {
            serde_json::Value::String(s) => Some(Utc.datetime_from_str(&s, "%+").ok()?),
            _ => None,
        }
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        Ok(self.to_rfc3339().into())
    }
}
