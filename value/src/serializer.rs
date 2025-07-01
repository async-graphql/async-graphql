use std::{error::Error, fmt};

use indexmap::IndexMap;
use serde::{
    Serialize,
    ser::{self, Impossible},
};

use crate::{ConstValue, Name, Number};

/// This type represents errors that can occur when serializing.
#[derive(Debug)]
pub struct SerializerError(String);

impl fmt::Display for SerializerError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SerializerError(ref s) => fmt.write_str(s),
        }
    }
}

impl Error for SerializerError {
    fn description(&self) -> &str {
        "ConstValue serializer error"
    }
}

impl ser::Error for SerializerError {
    fn custom<T: fmt::Display>(msg: T) -> SerializerError {
        SerializerError(msg.to_string())
    }
}

/// Convert a `T` into `ConstValue` which is an enum that can represent any
/// valid GraphQL data.
#[inline]
pub fn to_value<T: ser::Serialize>(value: T) -> Result<ConstValue, SerializerError> {
    value.serialize(Serializer)
}

struct Serializer;

impl ser::Serializer for Serializer {
    type Ok = ConstValue;
    type Error = SerializerError;
    type SerializeSeq = SerializeSeq;
    type SerializeTuple = SerializeTuple;
    type SerializeTupleStruct = SerializeTupleStruct;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeStruct;
    type SerializeStructVariant = SerializeStructVariant;

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(ConstValue::Boolean(v))
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(ConstValue::Number(v.into()))
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(ConstValue::Number(v.into()))
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(ConstValue::Number(v.into()))
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(ConstValue::Number(v.into()))
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(ConstValue::Number(v.into()))
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(ConstValue::Number(v.into()))
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(ConstValue::Number(v.into()))
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(ConstValue::Number(v.into()))
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        match Number::from_f64(v) {
            Some(v) => Ok(ConstValue::Number(v)),
            None => Ok(ConstValue::Null),
        }
    }

    #[inline]
    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(SerializerError("char is not supported.".to_string()))
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(ConstValue::String(v.to_string()))
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(ConstValue::Binary(v.to_vec().into()))
    }

    #[inline]
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(ConstValue::Null)
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize + ?Sized,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(ConstValue::Null)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(ConstValue::Null)
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(ConstValue::String(variant.to_string()))
    }

    #[inline]
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize + ?Sized,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize + ?Sized,
    {
        value.serialize(self).map(|v| {
            let mut map = IndexMap::new();
            map.insert(Name::new(variant), v);
            ConstValue::Object(map)
        })
    }

    #[inline]
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SerializeSeq(vec![]))
    }

    #[inline]
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(SerializeTuple(vec![]))
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(SerializeTupleStruct(vec![]))
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(SerializeTupleVariant(
            Name::new(variant),
            Vec::with_capacity(len),
        ))
    }

    #[inline]
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(SerializeMap {
            map: IndexMap::new(),
            key: None,
        })
    }

    #[inline]
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(SerializeStruct(IndexMap::new()))
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(SerializeStructVariant(Name::new(variant), IndexMap::new()))
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        true
    }
}

struct SerializeSeq(Vec<ConstValue>);

impl ser::SerializeSeq for SerializeSeq {
    type Ok = ConstValue;
    type Error = SerializerError;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize + ?Sized,
    {
        let value = value.serialize(Serializer)?;
        self.0.push(value);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(ConstValue::List(self.0))
    }
}

struct SerializeTuple(Vec<ConstValue>);

impl ser::SerializeTuple for SerializeTuple {
    type Ok = ConstValue;
    type Error = SerializerError;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize + ?Sized,
    {
        let value = value.serialize(Serializer)?;
        self.0.push(value);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(ConstValue::List(self.0))
    }
}

struct SerializeTupleStruct(Vec<ConstValue>);

impl ser::SerializeTupleStruct for SerializeTupleStruct {
    type Ok = ConstValue;
    type Error = SerializerError;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize + ?Sized,
    {
        let value = value.serialize(Serializer)?;
        self.0.push(value);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(ConstValue::List(self.0))
    }
}

struct SerializeTupleVariant(Name, Vec<ConstValue>);

impl ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = ConstValue;
    type Error = SerializerError;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize + ?Sized,
    {
        let value = value.serialize(Serializer)?;
        self.1.push(value);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        let mut map = IndexMap::new();
        map.insert(self.0, ConstValue::List(self.1));
        Ok(ConstValue::Object(map))
    }
}

struct SerializeMap {
    map: IndexMap<Name, ConstValue>,
    key: Option<Name>,
}

impl ser::SerializeMap for SerializeMap {
    type Ok = ConstValue;
    type Error = SerializerError;

    #[inline]
    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize + ?Sized,
    {
        let key = key.serialize(MapKeySerializer)?;
        self.key = Some(key);
        Ok(())
    }

    #[inline]
    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize + ?Sized,
    {
        let value = value.serialize(Serializer)?;
        self.map.insert(self.key.take().unwrap(), value);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(ConstValue::Object(self.map))
    }
}

struct SerializeStruct(IndexMap<Name, ConstValue>);

impl ser::SerializeStruct for SerializeStruct {
    type Ok = ConstValue;
    type Error = SerializerError;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize + ?Sized,
    {
        let key = Name::new(key);
        let value = value.serialize(Serializer)?;
        self.0.insert(key, value);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(ConstValue::Object(self.0))
    }
}

struct SerializeStructVariant(Name, IndexMap<Name, ConstValue>);

impl ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = ConstValue;
    type Error = SerializerError;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize + ?Sized,
    {
        let key = Name::new(key);
        let value = value.serialize(Serializer)?;
        self.1.insert(key, value);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        let mut map = IndexMap::new();
        map.insert(self.0, ConstValue::Object(self.1));
        Ok(ConstValue::Object(map))
    }
}

#[inline]
fn key_must_be_a_string() -> SerializerError {
    SerializerError("Key must be a string".to_string())
}

struct MapKeySerializer;

impl serde::Serializer for MapKeySerializer {
    type Ok = Name;
    type Error = SerializerError;
    type SerializeSeq = Impossible<Name, SerializerError>;
    type SerializeTuple = Impossible<Name, SerializerError>;
    type SerializeTupleStruct = Impossible<Name, SerializerError>;
    type SerializeTupleVariant = Impossible<Name, SerializerError>;
    type SerializeMap = Impossible<Name, SerializerError>;
    type SerializeStruct = Impossible<Name, SerializerError>;
    type SerializeStructVariant = Impossible<Name, SerializerError>;

    #[inline]
    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Name::new(v))
    }

    #[inline]
    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(Name::new(variant))
    }

    #[inline]
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(key_must_be_a_string())
    }
}
