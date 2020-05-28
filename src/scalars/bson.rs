use crate::{InputValueError, InputValueResult, ScalarType, Value};
use async_graphql_derive::Scalar;
use bson::{oid::ObjectId, UtcDateTime};
use chrono::{DateTime, Utc};

#[Scalar(internal)]
impl ScalarType for ObjectId {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => Ok(ObjectId::with_string(&s)?),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.to_string())
    }
}

#[Scalar(internal, name = "DateTime")]
impl ScalarType for UtcDateTime {
    fn parse(value: Value) -> InputValueResult<Self> {
        DateTime::<Utc>::parse(value).map(UtcDateTime::from)
    }

    fn to_value(&self) -> Value {
        (**self).to_value()
    }
}
