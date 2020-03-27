use crate::{impl_scalar_internal, Result, Scalar, Value};
use chrono_tz::Tz;
use std::str::FromStr;

impl Scalar for Tz {
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

impl_scalar_internal!(Tz);
