use crate::{Result, ScalarType, Value};
use async_graphql_derive::Scalar;
use bson::{oid::ObjectId, UtcDateTime};
use chrono::{DateTime, Utc};

#[Scalar(internal)]
impl ScalarType for ObjectId {
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

#[Scalar(internal)]
impl ScalarType for UtcDateTime {
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
