use crate::{impl_scalar_internal, Result, Scalar, Value};
use url::Url;

impl Scalar for Url {
    fn type_name() -> &'static str {
        "Url"
    }

    fn parse(value: &Value) -> Option<Self> {
        match value {
            Value::String(s) => Some(Url::parse(s).ok()?),
            _ => None,
        }
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        Ok(self.to_string().into())
    }
}

impl_scalar_internal!(Url);
