use crate::{impl_scalar, GQLScalar, Result, Value};
use uuid::Uuid;

impl GQLScalar for Uuid {
    fn type_name() -> &'static str {
        "UUID"
    }

    fn parse(value: &Value) -> Option<Self> {
        match value {
            Value::String(s) => Some(Uuid::parse_str(&s).ok()?),
            _ => None,
        }
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        Ok(self.to_string().into())
    }
}

impl_scalar!(Uuid);
