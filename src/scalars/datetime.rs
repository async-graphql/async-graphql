use crate::{GqlInputValueResult, GqlResult, GqlValue, InputValueError, ScalarType};
use async_graphql_derive::GqlScalar;
use chrono::{DateTime, TimeZone, Utc};

/// Implement the DateTime<Utc> scalar
///
/// The input/output is a string in RFC3339 format.
#[GqlScalar(internal)]
impl ScalarType for DateTime<Utc> {
    fn type_name() -> &'static str {
        "DateTime"
    }

    fn parse(value: GqlValue) -> GqlInputValueResult<Self> {
        match value {
            GqlValue::String(s) => Ok(Utc.datetime_from_str(&s, "%+")?),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn to_json(&self) -> GqlResult<serde_json::Value> {
        Ok(self.to_rfc3339().into())
    }
}
