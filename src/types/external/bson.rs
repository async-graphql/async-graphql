use crate::{InputValueError, InputValueResult, Scalar, ScalarType, Value};
use bson::oid::ObjectId;
#[cfg(feature = "chrono")]
use bson::DateTime as UtcDateTime;
#[cfg(feature = "chrono")]
use chrono::{DateTime, Utc};

#[Scalar(internal)]
impl ScalarType for ObjectId {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => Ok(ObjectId::with_string(&s)?),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.to_string())
    }
}

#[cfg(feature = "chrono")]
#[Scalar(internal, name = "DateTime")]
impl ScalarType for UtcDateTime {
    fn parse(value: Value) -> InputValueResult<Self> {
        <DateTime<Utc>>::parse(value)
            .map_err(InputValueError::propagate)
            .map(UtcDateTime::from)
    }

    fn to_value(&self) -> Value {
        (**self).to_value()
    }
}
