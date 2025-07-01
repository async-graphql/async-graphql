use std::fmt::{self, Formatter};

use indexmap::IndexMap;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error as DeError, MapAccess, SeqAccess, Visitor},
    ser::SerializeMap,
};

use crate::{ConstValue, Name, Number, Value};

/// The token used by `serde_json` to represent raw values.
///
/// It should be kept in sync with the following original until made public:
/// https://github.com/serde-rs/json/blob/b48b9a3a0c09952579e98c8940fe0d1ee4aae588/src/raw.rs#L292
#[cfg(feature = "raw_value")]
pub const RAW_VALUE_TOKEN: &str = "$serde_json::private::RawValue";

impl Serialize for ConstValue {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ConstValue::Null => serializer.serialize_none(),
            ConstValue::Number(v) => v.serialize(serializer),
            ConstValue::String(v) => serializer.serialize_str(v),
            ConstValue::Boolean(v) => serializer.serialize_bool(*v),
            ConstValue::Binary(v) => serializer.serialize_bytes(v),
            ConstValue::Enum(v) => serializer.serialize_str(v),
            ConstValue::List(v) => v.serialize(serializer),
            ConstValue::Object(v) => {
                #[cfg(feature = "raw_value")]
                if v.len() == 1 {
                    if let Some(ConstValue::String(v)) = v.get(RAW_VALUE_TOKEN) {
                        if let Ok(v) = serde_json::value::RawValue::from_string(v.clone()) {
                            return v.serialize(serializer);
                        }
                    }
                }
                v.serialize(serializer)
            }
        }
    }
}

impl<'de> Deserialize<'de> for ConstValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = ConstValue;

            #[inline]
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("any valid value")
            }

            #[inline]
            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(ConstValue::Boolean(v))
            }

            #[inline]
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(ConstValue::Number(v.into()))
            }

            #[inline]
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(ConstValue::Number(v.into()))
            }

            #[inline]
            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(Number::from_f64(v).map_or(ConstValue::Null, ConstValue::Number))
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(ConstValue::String(v.to_string()))
            }

            #[inline]
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(ConstValue::String(v))
            }

            #[inline]
            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(ConstValue::Binary(v.to_vec().into()))
            }

            #[inline]
            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(ConstValue::Binary(v.into()))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(ConstValue::Null)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(ConstValue::Null)
            }

            fn visit_seq<A>(self, mut visitor: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut vec = Vec::new();
                while let Some(elem) = visitor.next_element()? {
                    vec.push(elem);
                }
                Ok(ConstValue::List(vec))
            }

            fn visit_map<A>(self, mut visitor: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut map = IndexMap::new();
                while let Some((name, value)) = visitor.next_entry()? {
                    map.insert(name, value);
                }
                Ok(ConstValue::Object(map))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

#[derive(Debug)]
struct SerdeVariable(Name);

impl Serialize for SerdeVariable {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_map(Some(1))?;
        s.serialize_entry("$var", &self.0)?;
        s.end()
    }
}

impl Serialize for Value {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Value::Variable(name) => SerdeVariable(name.clone()).serialize(serializer),
            Value::Null => serializer.serialize_none(),
            Value::Number(v) => v.serialize(serializer),
            Value::String(v) => serializer.serialize_str(v),
            Value::Boolean(v) => serializer.serialize_bool(*v),
            Value::Binary(v) => serializer.serialize_bytes(v),
            Value::Enum(v) => serializer.serialize_str(v),
            Value::List(v) => v.serialize(serializer),
            Value::Object(v) => v.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;

            #[inline]
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("any valid value")
            }

            #[inline]
            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(Value::Boolean(v))
            }

            #[inline]
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(Value::Number(v.into()))
            }

            #[inline]
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(Value::Number(v.into()))
            }

            #[inline]
            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(Number::from_f64(v).map_or(Value::Null, Value::Number))
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(Value::String(v.to_string()))
            }

            #[inline]
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(Value::String(v))
            }

            #[inline]
            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(Value::Binary(v.to_vec().into()))
            }

            #[inline]
            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(Value::Binary(v.into()))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(Value::Null)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(Value::Null)
            }

            fn visit_seq<A>(self, mut visitor: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut vec = Vec::new();
                while let Some(elem) = visitor.next_element()? {
                    vec.push(elem);
                }
                Ok(Value::List(vec))
            }

            fn visit_map<A>(self, mut visitor: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut map = IndexMap::new();
                while let Some((name, value)) = visitor.next_entry()? {
                    match &value {
                        Value::String(value) if name == "$var" => {
                            return Ok(Value::Variable(Name::new(value)));
                        }
                        _ => {
                            map.insert(name, value);
                        }
                    }
                }
                Ok(Value::Object(map))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn var_serde() {
        let var = Value::Variable(Name::new("abc"));
        let s = serde_json::to_string(&var).unwrap();
        assert_eq!(s, r#"{"$var":"abc"}"#);
        assert_eq!(var, serde_json::from_str(&s).unwrap());
    }
}
