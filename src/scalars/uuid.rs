use crate::{GqlInputValueResult, GqlResult, GqlValue, InputValueError, ScalarType};
use async_graphql_derive::GqlScalar;
use uuid::Uuid;

#[GqlScalar(internal)]
impl ScalarType for Uuid {
    fn type_name() -> &'static str {
        "UUID"
    }

    fn parse(value: GqlValue) -> GqlInputValueResult<Self> {
        match value {
            GqlValue::String(s) => Ok(Uuid::parse_str(&s)?),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn to_json(&self) -> GqlResult<serde_json::Value> {
        Ok(self.to_string().into())
    }
}
