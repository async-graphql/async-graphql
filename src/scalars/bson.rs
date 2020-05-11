use crate::{GqlInputValueResult, GqlResult, GqlValue, InputValueError, ScalarType};
use async_graphql_derive::GqlScalar;
use bson::{oid::ObjectId, UtcDateTime};
use chrono::{DateTime, Utc};

#[GqlScalar(internal)]
impl ScalarType for ObjectId {
    fn type_name() -> &'static str {
        "ObjectId"
    }

    fn parse(value: GqlValue) -> GqlInputValueResult<Self> {
        match value {
            GqlValue::String(s) => Ok(ObjectId::with_string(&s)?),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn to_json(&self) -> GqlResult<serde_json::Value> {
        Ok(self.to_string().into())
    }
}

#[GqlScalar(internal)]
impl ScalarType for UtcDateTime {
    fn type_name() -> &'static str {
        "DateTime"
    }

    fn parse(value: GqlValue) -> GqlInputValueResult<Self> {
        DateTime::<Utc>::parse(value).map(UtcDateTime::from)
    }

    fn to_json(&self) -> GqlResult<serde_json::Value> {
        (**self).to_json()
    }
}
