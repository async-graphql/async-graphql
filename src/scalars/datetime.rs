use crate::{impl_scalar_internal, JsonWriter, Result, Scalar, Value};
use chrono::{DateTime, TimeZone, Utc};

/// Implement the DateTime<Utc> scalar
///
/// The input/output is a string in RFC3339 format.
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

    fn to_json(&self, w: &mut JsonWriter) -> Result<()> {
        w.string(&self.to_rfc3339());
        Ok(())
    }
}

impl_scalar_internal!(DateTime<Utc>);
