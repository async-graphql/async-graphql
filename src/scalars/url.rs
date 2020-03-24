use crate::{impl_scalar_internal, JsonWriter, Result, Scalar, Value};
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

    fn to_json(&self, w: &mut JsonWriter) -> Result<()> {
        w.string(&self.to_string());
        Ok(())
    }
}

impl_scalar_internal!(Url);
