use crate::{impl_scalar_internal, JsonWriter, Result, Scalar, Value};
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

    fn to_json(&self, w: &mut JsonWriter) -> Result<()> {
        w.string(&self.to_string());
        Ok(())
    }
}

impl_scalar_internal!(ObjectId);
