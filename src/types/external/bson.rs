#[cfg(feature = "chrono")]
use bson::DateTime as UtcDateTime;
use bson::{oid::ObjectId, Bson, Document, Uuid};
#[cfg(feature = "chrono")]
use chrono::{DateTime, Utc};

use crate::{InputValueError, InputValueResult, Scalar, ScalarType, Value};

#[Scalar(internal)]
impl ScalarType for ObjectId {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => Ok(ObjectId::parse_str(s)?),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.to_string())
    }
}

#[Scalar(internal, name = "UUID")]
impl ScalarType for Uuid {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => Ok(Uuid::parse_str(s)?),
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
            .map(UtcDateTime::from_chrono)
    }

    fn to_value(&self) -> Value {
        self.to_chrono().to_value()
    }
}

#[Scalar(internal, name = "JSON")]
impl ScalarType for Bson {
    fn parse(value: Value) -> InputValueResult<Self> {
        bson::to_bson(&value).map_err(InputValueError::custom)
    }

    fn to_value(&self) -> Value {
        bson::from_bson(self.clone()).unwrap_or_default()
    }
}

#[Scalar(internal, name = "JSONObject")]
impl ScalarType for Document {
    fn parse(value: Value) -> InputValueResult<Self> {
        bson::to_document(&value).map_err(InputValueError::custom)
    }

    fn to_value(&self) -> Value {
        bson::from_document(self.clone()).unwrap_or_default()
    }
}
