use crate::{ConstValue, Name};
use serde::de::{
    self, Deserialize, DeserializeOwned, DeserializeSeed, EnumAccess, Error as DeError,
    IntoDeserializer, MapAccess, SeqAccess, Unexpected, VariantAccess, Visitor,
};
use serde::forward_to_deserialize_any;
use std::collections::BTreeMap;
use std::{fmt, vec};

#[derive(Debug)]
pub enum DeserializerError {
    Custom(String),
}

impl de::Error for DeserializerError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        DeserializerError::Custom(msg.to_string())
    }
}

impl std::error::Error for DeserializerError {
    fn description(&self) -> &str {
        "Value deserializer error"
    }
}

impl fmt::Display for DeserializerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DeserializerError::Custom(ref msg) => write!(f, "{}", msg),
        }
    }
}

impl From<de::value::Error> for DeserializerError {
    fn from(e: de::value::Error) -> DeserializerError {
        DeserializerError::Custom(e.to_string())
    }
}

impl ConstValue {
    fn unexpected(&self) -> Unexpected {
        match self {
            ConstValue::Null => Unexpected::Unit,
            ConstValue::Number(_) => Unexpected::Other("number"),
            ConstValue::String(v) => Unexpected::Str(v),
            ConstValue::Boolean(v) => Unexpected::Bool(*v),
            ConstValue::Enum(v) => Unexpected::Str(v),
            ConstValue::List(_) => Unexpected::Seq,
            ConstValue::Object(_) => Unexpected::Map,
        }
    }
}

fn visit_array<'de, V>(array: Vec<ConstValue>, visitor: V) -> Result<V::Value, DeserializerError>
where
    V: Visitor<'de>,
{
    let len = array.len();
    let mut deserializer = SeqDeserializer::new(array);
    let seq = visitor.visit_seq(&mut deserializer)?;
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(seq)
    } else {
        Err(DeserializerError::invalid_length(
            len,
            &"fewer elements in array",
        ))
    }
}

fn visit_object<'de, V>(
    object: BTreeMap<Name, ConstValue>,
    visitor: V,
) -> Result<V::Value, DeserializerError>
where
    V: Visitor<'de>,
{
    let len = object.len();
    let mut deserializer = MapDeserializer::new(object);
    let map = visitor.visit_map(&mut deserializer)?;
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(map)
    } else {
        Err(DeserializerError::invalid_length(
            len,
            &"fewer elements in map",
        ))
    }
}

impl<'de> de::Deserializer<'de> for ConstValue {
    type Error = DeserializerError;

    fn deserialize_any<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ConstValue::Null => visitor.visit_unit(),
            ConstValue::Number(v) => v
                .deserialize_any(visitor)
                .map_err(|err| DeserializerError::Custom(err.to_string())),
            ConstValue::String(v) => visitor.visit_str(&v),
            ConstValue::Boolean(v) => visitor.visit_bool(v),
            ConstValue::Enum(v) => visitor.visit_str(v.as_str()),
            ConstValue::List(v) => visit_array(v, visitor),
            ConstValue::Object(v) => visit_object(v, visitor),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ConstValue::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (variant, value) = match self {
            ConstValue::Object(value) => {
                let mut iter = value.into_iter();
                let (variant, value) = match iter.next() {
                    Some(v) => v,
                    None => {
                        return Err(serde::de::Error::invalid_value(
                            Unexpected::Map,
                            &"map with a single key",
                        ));
                    }
                };
                // enums are encoded in json as maps with a single key:value pair
                if iter.next().is_some() {
                    return Err(serde::de::Error::invalid_value(
                        Unexpected::Map,
                        &"map with a single key",
                    ));
                }
                (variant, Some(value))
            }
            ConstValue::String(variant) => (Name::new(&variant), None),
            ConstValue::Enum(variant) => (variant, None),
            other => {
                return Err(DeserializerError::invalid_type(
                    other.unexpected(),
                    &"string or map",
                ));
            }
        };

        visitor.visit_enum(EnumDeserializer { variant, value })
    }
}

struct EnumDeserializer {
    variant: Name,
    value: Option<ConstValue>,
}

impl<'de> EnumAccess<'de> for EnumDeserializer {
    type Error = DeserializerError;
    type Variant = VariantDeserializer;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, VariantDeserializer), DeserializerError>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = self.variant.into_deserializer();
        let visitor = VariantDeserializer { value: self.value };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}

impl<'de> IntoDeserializer<'de, DeserializerError> for ConstValue {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

struct VariantDeserializer {
    value: Option<ConstValue>,
}

impl<'de> VariantAccess<'de> for VariantDeserializer {
    type Error = DeserializerError;

    fn unit_variant(self) -> Result<(), DeserializerError> {
        match self.value {
            Some(value) => Deserialize::deserialize(value),
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, DeserializerError>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value {
            Some(value) => seed.deserialize(value),
            None => Err(DeserializerError::invalid_type(
                Unexpected::UnitVariant,
                &"newtype variant",
            )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(ConstValue::List(v)) => {
                serde::Deserializer::deserialize_any(SeqDeserializer::new(v), visitor)
            }
            Some(other) => Err(serde::de::Error::invalid_type(
                other.unexpected(),
                &"tuple variant",
            )),
            None => Err(DeserializerError::invalid_type(
                Unexpected::UnitVariant,
                &"tuple variant",
            )),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(ConstValue::Object(v)) => {
                serde::Deserializer::deserialize_any(MapDeserializer::new(v), visitor)
            }
            Some(other) => Err(DeserializerError::invalid_type(
                other.unexpected(),
                &"struct variant",
            )),
            None => Err(DeserializerError::invalid_type(
                Unexpected::UnitVariant,
                &"struct variant",
            )),
        }
    }
}

struct SeqDeserializer {
    iter: vec::IntoIter<ConstValue>,
}

impl SeqDeserializer {
    fn new(vec: Vec<ConstValue>) -> Self {
        SeqDeserializer {
            iter: vec.into_iter(),
        }
    }
}

impl<'de> serde::Deserializer<'de> for SeqDeserializer {
    type Error = DeserializerError;

    #[inline]
    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        let len = self.iter.len();
        if len == 0 {
            visitor.visit_unit()
        } else {
            let ret = visitor.visit_seq(&mut self)?;
            let remaining = self.iter.len();
            if remaining == 0 {
                Ok(ret)
            } else {
                Err(DeserializerError::invalid_length(
                    len,
                    &"fewer elements in array",
                ))
            }
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> SeqAccess<'de> for SeqDeserializer {
    type Error = DeserializerError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, DeserializerError>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

struct MapDeserializer {
    iter: <BTreeMap<Name, ConstValue> as IntoIterator>::IntoIter,
    value: Option<ConstValue>,
}

impl MapDeserializer {
    fn new(map: BTreeMap<Name, ConstValue>) -> Self {
        MapDeserializer {
            iter: map.into_iter(),
            value: None,
        }
    }
}

impl<'de> MapAccess<'de> for MapDeserializer {
    type Error = DeserializerError;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, DeserializerError>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                let key_de = MapKeyDeserializer { key };
                seed.deserialize(key_de).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, DeserializerError>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => Err(serde::de::Error::custom("value is missing")),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

impl<'de> serde::Deserializer<'de> for MapDeserializer {
    type Error = DeserializerError;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct MapKeyDeserializer {
    key: Name,
}

impl<'de> serde::Deserializer<'de> for MapKeyDeserializer {
    type Error = DeserializerError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        NameDeserializer::new(self.key).deserialize_any(visitor)
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        self.key
            .into_deserializer()
            .deserialize_enum(name, variants, visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        bytes byte_buf unit unit_struct seq tuple option newtype_struct
        tuple_struct map struct identifier ignored_any
    }
}

struct NameDeserializer {
    value: Name,
}

impl NameDeserializer {
    fn new(value: Name) -> Self {
        NameDeserializer { value }
    }
}

impl<'de> de::Deserializer<'de> for NameDeserializer {
    type Error = DeserializerError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, DeserializerError>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_string(self.value.to_string())
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple enum
        tuple_struct map struct identifier ignored_any
    }
}

/// Interpret a `ConstValue` as an instance of type `T`.
pub fn from_value<T: DeserializeOwned>(value: ConstValue) -> Result<T, DeserializerError> {
    T::deserialize(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Number;
    use serde::Deserialize;
    use std::collections::HashMap;

    #[test]
    fn test_deserializer() {
        let n: bool = from_value(ConstValue::Boolean(true)).unwrap();
        assert_eq!(n, true);

        let n: i32 = from_value(ConstValue::Number(100i32.into())).unwrap();
        assert_eq!(n, 100);

        let n: f32 = from_value(ConstValue::Number(Number::from_f64(1.123f64).unwrap())).unwrap();
        assert_eq!(n, 1.123);

        let n: Option<i32> = from_value(ConstValue::Number(100i32.into())).unwrap();
        assert_eq!(n, Some(100));

        let n: Option<i32> = from_value(ConstValue::Null).unwrap();
        assert_eq!(n, None);

        let n: Vec<i32> = from_value(
            (0..10)
                .into_iter()
                .map(|v| ConstValue::Number(v.into()))
                .collect(),
        )
        .unwrap();
        assert_eq!(n, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

        #[derive(Deserialize)]
        struct NewType(i32);

        let n: NewType = from_value(ConstValue::Number(100i32.into())).unwrap();
        assert_eq!(n.0, 100);

        #[derive(Deserialize, Debug, Eq, PartialEq, Hash, Copy, Clone)]
        enum Enum {
            A,
            B,
        }
        let n: Enum = from_value(ConstValue::String("A".to_string())).unwrap();
        assert_eq!(n, Enum::A);

        let n: Enum = from_value(ConstValue::Enum(Name::new("B"))).unwrap();
        assert_eq!(n, Enum::B);

        let mut obj = BTreeMap::<Name, ConstValue>::new();
        obj.insert(Name::new("A"), ConstValue::Number(10.into()));
        obj.insert(Name::new("B"), ConstValue::Number(20.into()));
        let n: HashMap<Enum, i32> = from_value(ConstValue::Object(obj)).unwrap();
        assert_eq!(10, n[&Enum::A]);
        assert_eq!(20, n[&Enum::B]);

        #[derive(Deserialize, Debug, Eq, PartialEq)]
        struct Struct {
            a: i32,
            b: Option<Enum>,
        }
        let mut obj = BTreeMap::<Name, ConstValue>::new();
        obj.insert(Name::new("a"), ConstValue::Number(10.into()));
        obj.insert(Name::new("b"), ConstValue::Enum(Name::new("B")));
        let n: Struct = from_value(ConstValue::Object(obj)).unwrap();
        assert_eq!(
            n,
            Struct {
                a: 10,
                b: Some(Enum::B)
            }
        );
    }
}
