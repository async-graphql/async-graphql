use crate::{GqlInputValueResult, GqlResult, GqlValue, InputValueError, ScalarType};
use async_graphql_derive::GqlScalar;
use chrono_tz::Tz;
use std::str::FromStr;

#[GqlScalar(internal)]
impl ScalarType for Tz {
    fn type_name() -> &'static str {
        "TimeZone"
    }

    fn parse(value: GqlValue) -> GqlInputValueResult<Self> {
        match value {
            GqlValue::String(s) => Ok(Tz::from_str(&s)?),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn to_json(&self) -> GqlResult<serde_json::Value> {
        Ok(Tz::name(self).into())
    }
}
