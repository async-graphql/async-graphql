use std::borrow::Cow;

use indexmap::IndexMap;
use serde::de::DeserializeOwned;

use crate::{Error, Name, Result, Upload, Value};

/// A value accessor
pub struct ValueAccessor<'a>(&'a Value);

impl<'a> ValueAccessor<'a> {
    /// Returns `true` if the value is null, otherwise returns `false`
    #[inline]
    pub fn is_null(&self) -> bool {
        matches!(self.0, Value::Null)
    }

    /// Returns the boolean
    pub fn boolean(&self) -> Result<bool> {
        match self.0 {
            Value::Boolean(b) => Ok(*b),
            _ => Err(Error::new("internal: not a boolean")),
        }
    }

    /// Returns the enum name
    pub fn enum_name(&self) -> Result<&str> {
        match self.0 {
            Value::Enum(s) => Ok(s),
            Value::String(s) => Ok(s.as_str()),
            _ => Err(Error::new("internal: not an enum name")),
        }
    }

    /// Returns the number as `i64`
    pub fn i64(&self) -> Result<i64> {
        if let Value::Number(number) = self.0 {
            if let Some(value) = number.as_i64() {
                return Ok(value);
            }
        }
        Err(Error::new("internal: not an signed integer"))
    }

    /// Returns the number as `u64`
    pub fn u64(&self) -> Result<u64> {
        if let Value::Number(number) = self.0 {
            if let Some(value) = number.as_u64() {
                return Ok(value);
            }
        }
        Err(Error::new("internal: not an unsigned integer"))
    }

    /// Returns the number as `f32`
    pub fn f32(&self) -> Result<f32> {
        if let Value::Number(number) = self.0 {
            if let Some(value) = number.as_f64() {
                return Ok(value as f32);
            }
        }
        Err(Error::new("internal: not a float"))
    }

    /// Returns the number as `f64`
    pub fn f64(&self) -> Result<f64> {
        if let Value::Number(number) = self.0 {
            if let Some(value) = number.as_f64() {
                return Ok(value);
            }
        }
        Err(Error::new("internal: not a float"))
    }

    /// Returns the string value
    pub fn string(&self) -> Result<&'a str> {
        if let Value::String(value) = self.0 {
            Ok(value)
        } else {
            Err(Error::new("internal: not a string"))
        }
    }

    /// Returns the object accessor
    pub fn object(&self) -> Result<ObjectAccessor<'a>> {
        if let Value::Object(obj) = self.0 {
            Ok(ObjectAccessor(Cow::Borrowed(obj)))
        } else {
            Err(Error::new("internal: not an object"))
        }
    }

    /// Returns the list accessor
    pub fn list(&self) -> Result<ListAccessor<'a>> {
        if let Value::List(list) = self.0 {
            Ok(ListAccessor(list))
        } else {
            Err(Error::new("internal: not a list"))
        }
    }

    /// Deserialize the value to `T`
    pub fn deserialize<T: DeserializeOwned>(&self) -> Result<T> {
        T::deserialize(self.0.clone()).map_err(|err| format!("internal: {}", err).into())
    }

    /// Returns a reference to the underlying `Value`
    #[inline]
    pub fn as_value(&self) -> &'a Value {
        self.0
    }

    /// Returns a upload object
    pub fn upload(&self) -> Result<Upload> {
        <Upload as crate::InputType>::parse(Some(self.0.clone()))
            .map_err(|_| Error::new("internal: not a upload"))
    }
}

/// A object accessor
pub struct ObjectAccessor<'a>(pub(crate) Cow<'a, IndexMap<Name, Value>>);

impl<'a> ObjectAccessor<'a> {
    /// Return a reference to the value stored for `key`, if it is present,
    /// else `None`.
    #[inline]
    pub fn get(&self, name: &str) -> Option<ValueAccessor<'_>> {
        self.0.get(name).map(ValueAccessor)
    }

    /// Like [`ObjectAccessor::get`], returns `Err` if the index does not exist
    #[inline]
    pub fn try_get(&self, name: &str) -> Result<ValueAccessor<'_>> {
        self.0
            .get(name)
            .map(ValueAccessor)
            .ok_or_else(|| Error::new(format!("internal: key \"{}\" not found", name)))
    }

    /// Return an iterator over the key-value pairs of the object, in their
    /// order
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&Name, ValueAccessor<'_>)> + '_ {
        self.0
            .iter()
            .map(|(name, value)| (name, ValueAccessor(value)))
    }

    /// Return an iterator over the keys of the object, in their order
    #[inline]
    pub fn keys(&self) -> impl Iterator<Item = &Name> + '_ {
        self.0.keys()
    }

    /// Return an iterator over the values of the object, in their order
    #[inline]
    pub fn values(&self) -> impl Iterator<Item = ValueAccessor<'_>> + '_ {
        self.0.values().map(ValueAccessor)
    }

    /// Returns the number of elements in the object
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the object has no members
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to the underlying IndexMap
    #[inline]
    pub fn as_index_map(&'a self) -> &'a IndexMap<Name, Value> {
        &self.0
    }
}

/// A list accessor
pub struct ListAccessor<'a>(pub(crate) &'a [Value]);

impl<'a> ListAccessor<'a> {
    /// Returns the number of elements in the list
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the list has a length of 0
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an iterator over the list
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = ValueAccessor<'_>> + '_ {
        self.0.iter().map(ValueAccessor)
    }

    /// Returns a reference to an element depending on the index
    #[inline]
    pub fn get(&self, idx: usize) -> Option<ValueAccessor<'_>> {
        self.0.get(idx).map(ValueAccessor)
    }

    /// Like [`ListAccessor::get`], returns `Err` if the index does not exist
    #[inline]
    pub fn try_get(&self, idx: usize) -> Result<ValueAccessor<'_>> {
        self.get(idx)
            .ok_or_else(|| Error::new(format!("internal: index \"{}\" not found", idx)))
    }

    /// Returns a new ListAccessor that represents a slice of the original
    #[inline]
    pub fn as_slice(&self, start: usize, end: usize) -> Result<ListAccessor<'a>> {
        if start <= end && end <= self.len() {
            Ok(ListAccessor(&self.0[start..end]))
        } else {
            Err(Error::new("internal: invalid slice indices"))
        }
    }

    /// Returns a reference to the underlying `&[Value]`
    #[inline]
    pub fn as_values_slice(&self) -> &'a [Value] {
        self.0
    }
}
