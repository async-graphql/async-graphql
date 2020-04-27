use crate::{Result, ScalarType, Value};
use async_graphql_derive::Scalar;
use url::Url;

#[Scalar(internal)]
impl ScalarType for Url {
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
