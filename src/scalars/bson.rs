use crate::{impl_scalar_internal, Result, Scalar, Value};
use bson::{oid::ObjectId, UtcDateTime};
use chrono::{DateTime, Utc};

impl Scalar for ObjectId {
    fn type_name() -> &'static str {
        "ObjectId"
    }

    fn parse(value: &Value) -> Option<Self> {
        match value {
            Value::String(s) => Some(ObjectId::with_string(&s).ok()?),
            _ => None,
        }
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        Ok(self.to_string().into())
    }
}

impl_scalar_internal!(ObjectId);

impl Scalar for UtcDateTime {
    fn type_name() -> &'static str {
        "DateTime"
    }

    fn parse(value: &Value) -> Option<Self> {
        DateTime::<Utc>::parse(value).map(UtcDateTime::from)
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        (**self).to_json()
    }
}

impl_scalar_internal!(UtcDateTime);
