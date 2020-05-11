use crate::{GqlInputValueResult, GqlResult, GqlValue, InputValueError, ScalarType};
use async_graphql_derive::GqlScalar;
use url::Url;

#[GqlScalar(internal)]
impl ScalarType for Url {
    fn type_name() -> &'static str {
        "Url"
    }

    fn parse(value: GqlValue) -> GqlInputValueResult<Self> {
        match value {
            GqlValue::String(s) => Ok(Url::parse(&s)?),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn to_json(&self) -> GqlResult<serde_json::Value> {
        Ok(self.to_string().into())
    }
}
