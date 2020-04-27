use crate::{Result, ScalarType, Value};
use async_graphql_derive::Scalar;
use chrono_tz::Tz;
use std::str::FromStr;

#[Scalar(internal)]
impl ScalarType for Tz {
    fn type_name() -> &'static str {
        "TimeZone"
    }

    fn parse(value: &Value) -> Option<Self> {
        match value {
            Value::String(s) => Some(Tz::from_str(&s).ok()?),
            _ => None,
        }
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        Ok(Tz::name(self).into())
    }
}
