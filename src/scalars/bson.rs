use crate::{impl_scalar_internal, Scalar, Result, Value};
use bson::oid::ObjectId;

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
