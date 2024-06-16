#[cfg(feature = "chrono")]
use bson::DateTime as UtcDateTime;
use bson::{oid::ObjectId, Bson, Document, Uuid};
#[cfg(feature = "chrono")]
use chrono::{DateTime, Utc};

use crate::{InputValueError, InputValueResult, Scalar, ScalarType, Value};
use base64::Engine;

#[Scalar(internal)]
impl ScalarType for ObjectId {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => Ok(ObjectId::parse_str(s)?),
            Value::Object(mut o) => {
                if o.len() > 1 {
                    return Err(InputValueError::custom(
                        "too many keys to be extended json representation of a BSON ObjectId",
                    ));
                }
                let Some(v) = o.shift_remove("$oid") else {
                    return Err(InputValueError::custom("missing field \"$oid\""));
                };
                let Value::String(s) = v else {
                    return Err(InputValueError::expected_type(v));
                };
                Ok(ObjectId::parse_str(s)?)
            }
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
            Value::Object(mut o) => {
                if o.len() > 1 {
                    return Err(InputValueError::custom(
                        "too many keys to be extended json representation of a BSON Uuid",
                    ));
                }
                let Some(v) = o.shift_remove("$binary") else {
                    return Err(InputValueError::custom("missing field \"$binary\""));
                };
                let Value::Object(mut o) = v else {
                    return Err(InputValueError::expected_type(v));
                };
                if o.len() > 2 {
                    return Err(InputValueError::custom(
                        "too many keys to be extended json representation of a BSON Uuid",
                    ));
                }
                let Some(t) = o.shift_remove("subType") else {
                    return Err(InputValueError::custom("missing field \"subType\""));
                };
                let Value::String(t) = t else {
                    return Err(InputValueError::expected_type(t));
                };
                let Ok(t) = u8::from_str_radix(&t, 16) else {
                    return Err(InputValueError::custom(
                        "could not decode subType from hex string",
                    ));
                };
                if t != <bson::spec::BinarySubtype as Into<u8>>::into(
                    bson::spec::BinarySubtype::Uuid,
                ) || t
                    != <bson::spec::BinarySubtype as Into<u8>>::into(
                        bson::spec::BinarySubtype::UuidOld,
                    )
                {
                    return Err(InputValueError::custom(
                        "wrong BSON binary subtype to be a BSON Uuid",
                    ));
                }
                let Some(payload) = o.shift_remove("base64") else {
                    return Err(InputValueError::custom("missing field \"base64\""));
                };
                let Value::String(s) = payload else {
                    return Err(InputValueError::expected_type(payload));
                };
                let Ok(payload_bytes) = base64::prelude::BASE64_STANDARD.decode(&s) else {
                    return Err(InputValueError::custom("could not decode payload"));
                };
                let Ok(payload_bytes) = <Vec<u8> as TryInto<[u8; 16]>>::try_into(payload_bytes)
                else {
                    return Err(InputValueError::custom("wrong number of payload bytes"));
                };
                Ok(Uuid::from_bytes(payload_bytes))
            }
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn parse_uuid() {
        let id = Uuid::new();
        let bson_value = bson::bson!(id);
        let extended_json_value = json!(bson_value);
        let gql_value = Value::from_json(extended_json_value).expect("valid json");
        assert_eq!(
            id,
            <Uuid as ScalarType>::parse(gql_value).expect("parsing succeeds")
        );
    }
}
