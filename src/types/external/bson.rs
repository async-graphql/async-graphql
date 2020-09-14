use crate::{GQLScalar, InputValueError, InputValueResult, ScalarType, Value};
use bson::oid::ObjectId;
#[cfg(feature = "chrono")]
use bson::DateTime as UtcDateTime;
#[cfg(feature = "chrono")]
use chrono::{DateTime, Utc};

#[GQLScalar(internal)]
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

#[cfg(feature = "chrono")]
#[GQLScalar(internal, name = "DateTime")]
impl ScalarType for UtcDateTime {
    fn parse(value: Value) -> InputValueResult<Self> {
        DateTime::<Utc>::parse(value).map(UtcDateTime::from)
    }

    fn to_value(&self) -> Value {
        (**self).to_value()
    }
}
