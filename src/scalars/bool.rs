use crate::{GqlInputValueResult, GqlResult, GqlValue, InputValueError, ScalarType};
use async_graphql_derive::GqlScalar;

#[GqlScalar(internal)]
impl ScalarType for bool {
    fn type_name() -> &'static str {
        "Boolean"
    }

    fn description() -> Option<&'static str> {
        Some("The `Boolean` scalar type represents `true` or `false`.")
    }

    fn parse(value: GqlValue) -> GqlInputValueResult<Self> {
        match value {
            GqlValue::Boolean(n) => Ok(n),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn is_valid(value: &GqlValue) -> bool {
        match value {
            GqlValue::Boolean(_) => true,
            _ => false,
        }
    }

    fn to_json(&self) -> GqlResult<serde_json::Value> {
        Ok((*self).into())
    }
}
