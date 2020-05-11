use crate::{InputValueError, InputValueResult, Result, ScalarType, Value};
use async_graphql_derive::Scalar;
use bson::{oid::ObjectId, UtcDateTime};
use chrono::{DateTime, Utc};

#[Scalar(internal)]
impl ScalarType for ObjectId {
    fn type_name() -> &'static str {
        "ObjectId"
    }

    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => Ok(ObjectId::with_string(&s)?),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        Ok(self.to_string().into())
    }
}

#[Scalar(internal)]
impl ScalarType for UtcDateTime {
    fn type_name() -> &'static str {
        "DateTime"
    }

    fn parse(value: Value) -> InputValueResult<Self> {
        DateTime::<Utc>::parse(value).map(UtcDateTime::from)
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        (**self).to_json()
    }
}
