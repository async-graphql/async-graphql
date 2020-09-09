//! GraphQL types.
//!
//! The two root types are [`ExecutableDocument`](struct.ExecutableDocument.html) and
//! [`ServiceDocument`](struct.ServiceDocument.html), representing an executable GraphQL query and a
//! GraphQL service respectively.
//!
//! This follows the [June 2018 edition of the GraphQL spec](https://spec.graphql.org/June2018/).

use crate::pos::{Pos, Positioned};
use serde::de::{Deserializer, Error as _, Unexpected};
use serde::ser::{Error as _, Serializer};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::collections::{BTreeMap, HashMap};
use std::convert::{TryFrom, TryInto};
use std::fmt::{self, Display, Formatter, Write};
use std::fs::File;
use std::ops::Deref;

pub use executable::*;
pub use serde_json::Number;
pub use service::*;

mod executable;
mod service;

/// The type of an operation; `query`, `mutation` or `subscription`.
///
/// [Reference](https://spec.graphql.org/June2018/#OperationType).
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum OperationType {
    /// A query.
    Query,
    /// A mutation.
    Mutation,
    /// A subscription.
    Subscription,
}

impl Display for OperationType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Query => "query",
            Self::Mutation => "mutation",
            Self::Subscription => "subscription",
        })
    }
}

/// A GraphQL type, for example `String` or `[String!]!`.
///
/// [Reference](https://spec.graphql.org/June2018/#Type).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Type {
    /// The base type.
    pub base: BaseType,
    /// Whether the type is nullable.
    pub nullable: bool,
}

impl Type {
    /// Create a type from the type string.
    #[must_use]
    pub fn new(ty: &str) -> Option<Self> {
        let (nullable, ty) = if let Some(rest) = ty.strip_suffix('!') {
            (false, rest)
        } else {
            (true, ty)
        };

        Some(Self {
            base: if let Some(ty) = ty.strip_prefix('[') {
                BaseType::List(Box::new(Self::new(ty.strip_suffix(']')?)?))
            } else {
                BaseType::Named(Name::new(ty.to_owned()).ok()?)
            },
            nullable,
        })
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.base.fmt(f)?;
        if !self.nullable {
            f.write_char('!')?;
        }
        Ok(())
    }
}

/// A GraphQL base type, for example `String` or `[String!]`. This does not include whether the
/// type is nullable; for that see [Type](struct.Type.html).
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BaseType {
    /// A named type, such as `String`.
    Named(Name),
    /// A list type, such as `[String]`.
    List(Box<Type>),
}

impl Display for BaseType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Named(name) => f.write_str(name),
            Self::List(ty) => write!(f, "[{}]", ty),
        }
    }
}

/// A resolved GraphQL value, for example `1` or `"Hello World!"`.
///
/// It can be serialized and deserialized. Enums will be converted to strings. Attempting to
/// serialize `Upload` will fail, and `Enum` and `Upload` cannot be deserialized.
///
/// [Reference](https://spec.graphql.org/June2018/#Value).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConstValue {
    /// `null`.
    Null,
    /// A number.
    Number(Number),
    /// A string.
    String(String),
    /// A boolean.
    Boolean(bool),
    /// An enum. These are typically in `SCREAMING_SNAKE_CASE`.
    #[serde(skip_deserializing)]
    Enum(Name),
    /// A list of values.
    List(Vec<ConstValue>),
    /// An object. This is a map of keys to values.
    Object(BTreeMap<Name, ConstValue>),
    /// An uploaded file.
    #[serde(serialize_with = "fail_serialize_upload", skip_deserializing)]
    Upload(UploadValue),
}

impl ConstValue {
    /// Convert this `ConstValue` into a `Value`.
    #[must_use]
    pub fn into_value(self) -> Value {
        match self {
            Self::Null => Value::Null,
            Self::Number(num) => Value::Number(num),
            Self::String(s) => Value::String(s),
            Self::Boolean(b) => Value::Boolean(b),
            Self::Enum(v) => Value::Enum(v),
            Self::List(items) => {
                Value::List(items.into_iter().map(ConstValue::into_value).collect())
            }
            Self::Object(map) => Value::Object(
                map.into_iter()
                    .map(|(key, value)| (key, value.into_value()))
                    .collect(),
            ),
            Self::Upload(upload) => Value::Upload(upload),
        }
    }

    /// Attempt to convert the value into JSON. This is equivalent to the `TryFrom` implementation.
    ///
    /// # Errors
    ///
    /// Fails if serialization fails (see enum docs for more info).
    pub fn into_json(self) -> serde_json::Result<serde_json::Value> {
        self.try_into()
    }

    /// Attempt to convert JSON into a value. This is equivalent to the `TryFrom` implementation.
    ///
    /// # Errors
    ///
    /// Fails if deserialization fails (see enum docs for more info).
    pub fn from_json(json: serde_json::Value) -> serde_json::Result<Self> {
        json.try_into()
    }
}

impl Default for ConstValue {
    fn default() -> Self {
        Self::Null
    }
}

impl Display for ConstValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(num) => write!(f, "{}", *num),
            Self::String(val) => write_quoted(val, f),
            Self::Boolean(true) => f.write_str("true"),
            Self::Boolean(false) => f.write_str("false"),
            Self::Null | Self::Upload(_) => f.write_str("null"),
            Self::Enum(name) => f.write_str(name),
            Self::List(items) => write_list(items, f),
            Self::Object(map) => write_object(map, f),
        }
    }
}

impl TryFrom<serde_json::Value> for ConstValue {
    type Error = serde_json::Error;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        Self::deserialize(value)
    }
}
impl TryFrom<ConstValue> for serde_json::Value {
    type Error = serde_json::Error;
    fn try_from(value: ConstValue) -> Result<Self, Self::Error> {
        serde_json::to_value(value)
    }
}

/// A GraphQL value, for example `1`, `$name` or `"Hello World!"`. This is
/// [`ConstValue`](enum.ConstValue.html) with variables.
///
/// It can be serialized and deserialized. Enums will be converted to strings. Attempting to
/// serialize `Upload` or `Variable` will fail, and `Enum`, `Upload` and `Variable` cannot be
/// deserialized.
///
/// [Reference](https://spec.graphql.org/June2018/#Value).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Value {
    /// A variable, without the `$`.
    #[serde(serialize_with = "fail_serialize_variable", skip_deserializing)]
    Variable(Name),
    /// `null`.
    Null,
    /// A number.
    Number(Number),
    /// A string.
    String(String),
    /// A boolean.
    Boolean(bool),
    /// An enum. These are typically in `SCREAMING_SNAKE_CASE`.
    #[serde(skip_deserializing)]
    Enum(Name),
    /// A list of values.
    List(Vec<Value>),
    /// An object. This is a map of keys to values.
    Object(BTreeMap<Name, Value>),
    /// An uploaded file.
    #[serde(serialize_with = "fail_serialize_upload", skip_deserializing)]
    Upload(UploadValue),
}

impl Value {
    /// Attempt to convert the value into a const value by using a function to get a variable.
    pub fn into_const_with<E>(
        self,
        mut f: impl FnMut(Name) -> Result<ConstValue, E>,
    ) -> Result<ConstValue, E> {
        self.into_const_with_mut(&mut f)
    }

    fn into_const_with_mut<E>(
        self,
        f: &mut impl FnMut(Name) -> Result<ConstValue, E>,
    ) -> Result<ConstValue, E> {
        Ok(match self {
            Self::Variable(name) => f(name)?,
            Self::Null => ConstValue::Null,
            Self::Number(num) => ConstValue::Number(num),
            Self::String(s) => ConstValue::String(s),
            Self::Boolean(b) => ConstValue::Boolean(b),
            Self::Enum(v) => ConstValue::Enum(v),
            Self::List(items) => ConstValue::List(
                items
                    .into_iter()
                    .map(|value| value.into_const_with_mut(f))
                    .collect::<Result<_, _>>()?,
            ),
            Self::Object(map) => ConstValue::Object(
                map.into_iter()
                    .map(|(key, value)| Ok((key, value.into_const_with_mut(f)?)))
                    .collect::<Result<_, _>>()?,
            ),
            Self::Upload(upload) => ConstValue::Upload(upload),
        })
    }

    /// Attempt to convert the value into a const value.
    ///
    /// Will fail if the value contains variables.
    #[must_use]
    pub fn into_const(self) -> Option<ConstValue> {
        self.into_const_with(|_| Err(())).ok()
    }

    /// Attempt to convert the value into JSON. This is equivalent to the `TryFrom` implementation.
    ///
    /// # Errors
    ///
    /// Fails if serialization fails (see enum docs for more info).
    pub fn into_json(self) -> serde_json::Result<serde_json::Value> {
        self.try_into()
    }

    /// Attempt to convert JSON into a value. This is equivalent to the `TryFrom` implementation.
    ///
    /// # Errors
    ///
    /// Fails if deserialization fails (see enum docs for more info).
    pub fn from_json(json: serde_json::Value) -> serde_json::Result<Self> {
        json.try_into()
    }
}

impl Default for Value {
    fn default() -> Self {
        Self::Null
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Variable(name) => write!(f, "${}", name),
            Self::Number(num) => write!(f, "{}", *num),
            Self::String(val) => write_quoted(val, f),
            Self::Boolean(true) => f.write_str("true"),
            Self::Boolean(false) => f.write_str("false"),
            Self::Null | Self::Upload(_) => f.write_str("null"),
            Self::Enum(name) => f.write_str(name),
            Self::List(items) => write_list(items, f),
            Self::Object(map) => write_object(map, f),
        }
    }
}

impl From<ConstValue> for Value {
    fn from(value: ConstValue) -> Self {
        value.into_value()
    }
}

impl TryFrom<serde_json::Value> for Value {
    type Error = serde_json::Error;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        Self::deserialize(value)
    }
}
impl TryFrom<Value> for serde_json::Value {
    type Error = serde_json::Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        serde_json::to_value(value)
    }
}

fn fail_serialize_variable<S: Serializer>(_: &str, _: S) -> Result<S::Ok, S::Error> {
    Err(S::Error::custom("cannot serialize variable"))
}
fn fail_serialize_upload<S: Serializer>(_: &UploadValue, _: S) -> Result<S::Ok, S::Error> {
    Err(S::Error::custom("cannot serialize uploaded file"))
}

fn write_quoted(s: &str, f: &mut Formatter<'_>) -> fmt::Result {
    f.write_char('"')?;
    for c in s.chars() {
        match c {
            '\r' => f.write_str("\\r"),
            '\n' => f.write_str("\\n"),
            '\t' => f.write_str("\\t"),
            '"' => f.write_str("\\\""),
            '\\' => f.write_str("\\\\"),
            c if c.is_control() => write!(f, "\\u{:04}", c as u32),
            c => f.write_char(c),
        }?
    }
    f.write_char('"')
}
fn write_list<T: Display>(list: impl IntoIterator<Item = T>, f: &mut Formatter<'_>) -> fmt::Result {
    f.write_char('[')?;
    for item in list {
        item.fmt(f)?;
        f.write_char(',')?;
    }
    f.write_char(']')
}
fn write_object<K: Display, V: Display>(
    object: impl IntoIterator<Item = (K, V)>,
    f: &mut Formatter<'_>,
) -> fmt::Result {
    f.write_char('{')?;
    for (name, value) in object {
        write!(f, "{}: {},", name, value)?;
    }
    f.write_char('}')
}

/// A file upload value.
pub struct UploadValue {
    /// The name of the file.
    pub filename: String,
    /// The content type of the file.
    pub content_type: Option<String>,
    /// The file data.
    pub content: File,
}

impl UploadValue {
    /// Attempt to clone the upload value. This type's `Clone` implementation simply calls this and
    /// panics on failure.
    ///
    /// # Errors
    ///
    /// Fails if cloning the inner `File` fails.
    pub fn try_clone(&self) -> std::io::Result<Self> {
        Ok(Self {
            filename: self.filename.clone(),
            content_type: self.content_type.clone(),
            content: self.content.try_clone()?,
        })
    }
}

impl Clone for UploadValue {
    fn clone(&self) -> Self {
        self.try_clone().unwrap()
    }
}

impl fmt::Debug for UploadValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Upload({})", self.filename)
    }
}

impl PartialEq for UploadValue {
    fn eq(&self, other: &Self) -> bool {
        self.filename == other.filename
    }
}
impl Eq for UploadValue {}

/// A const GraphQL directive, such as `@deprecated(reason: "Use the other field)`. This differs
/// from [`Directive`](struct.Directive.html) in that it uses [`ConstValue`](enum.ConstValue.html)
/// instead of [`Value`](enum.Value.html).
///
/// [Reference](https://spec.graphql.org/June2018/#Directive).
#[derive(Debug, Clone)]
pub struct ConstDirective {
    /// The name of the directive.
    pub name: Positioned<Name>,
    /// The arguments to the directive.
    pub arguments: Vec<(Positioned<Name>, Positioned<ConstValue>)>,
}

impl ConstDirective {
    /// Convert this `ConstDirective` into a `Directive`.
    #[must_use]
    pub fn into_directive(self) -> Directive {
        Directive {
            name: self.name,
            arguments: self
                .arguments
                .into_iter()
                .map(|(name, value)| (name, value.map(ConstValue::into_value)))
                .collect(),
        }
    }

    /// Get the argument with the given name.
    #[must_use]
    pub fn get_argument(&self, name: &str) -> Option<&Positioned<ConstValue>> {
        self.arguments
            .iter()
            .find(|item| item.0.node == name)
            .map(|item| &item.1)
    }
}

/// A GraphQL directive, such as `@deprecated(reason: "Use the other field")`.
///
/// [Reference](https://spec.graphql.org/June2018/#Directive).
#[derive(Debug, Clone)]
pub struct Directive {
    /// The name of the directive.
    pub name: Positioned<Name>,
    /// The arguments to the directive.
    pub arguments: Vec<(Positioned<Name>, Positioned<Value>)>,
}

impl Directive {
    /// Attempt to convert this `Directive` into a `ConstDirective`.
    #[must_use]
    pub fn into_const(self) -> Option<ConstDirective> {
        Some(ConstDirective {
            name: self.name,
            arguments: self
                .arguments
                .into_iter()
                .map(|(name, value)| {
                    Some((name, Positioned::new(value.node.into_const()?, value.pos)))
                })
                .collect::<Option<_>>()?,
        })
    }

    /// Get the argument with the given name.
    #[must_use]
    pub fn get_argument(&self, name: &str) -> Option<&Positioned<Value>> {
        self.arguments
            .iter()
            .find(|item| item.0.node == name)
            .map(|item| &item.1)
    }
}

/// A GraphQL name. This is a newtype wrapper around a string with the addition guarantee that it
/// is a valid GraphQL name (follows the regex `[_A-Za-z][_0-9A-Za-z]*`).
///
/// [Reference](https://spec.graphql.org/June2018/#Name).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct Name(String);

impl Name {
    /// Check whether the name is valid (follows the regex `[_A-Za-z][_0-9A-Za-z]*`).
    #[must_use]
    pub fn is_valid(name: &str) -> bool {
        let mut bytes = name.bytes();
        bytes
            .next()
            .map_or(false, |c| c.is_ascii_alphabetic() || c == b'_')
            && bytes.all(|c| c.is_ascii_alphanumeric() || c == b'_')
    }

    /// Create a new name without checking whether it is valid or not. This will always check in
    /// debug mode.
    ///
    /// This function is not `unsafe` because an invalid name does not cause UB, but care should be
    /// taken to make sure it is a valid name.
    #[must_use]
    pub fn new_unchecked(name: String) -> Self {
        debug_assert!(Self::is_valid(&name));
        Self(name)
    }

    /// Create a new name, checking whether it is valid. Returns ownership of the string if it
    /// fails.
    ///
    /// # Errors
    ///
    /// Fails if the name is not a valid name.
    pub fn new(name: String) -> Result<Self, String> {
        if Self::is_valid(&name) {
            Ok(Self(name))
        } else {
            Err(name)
        }
    }

    /// Get the name as a string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert the name to a `String`.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for Name {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl From<Name> for String {
    fn from(name: Name) -> Self {
        name.0
    }
}

impl Deref for Name {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl PartialEq<String> for Name {
    fn eq(&self, other: &String) -> bool {
        self.0 == *other
    }
}
impl PartialEq<str> for Name {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}
impl PartialEq<Name> for String {
    fn eq(&self, other: &Name) -> bool {
        other == self
    }
}
impl PartialEq<Name> for str {
    fn eq(&self, other: &Name) -> bool {
        other == self
    }
}
impl<'a> PartialEq<&'a str> for Name {
    fn eq(&self, other: &&'a str) -> bool {
        self == *other
    }
}
impl<'a> PartialEq<Name> for &'a str {
    fn eq(&self, other: &Name) -> bool {
        other == self
    }
}

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(String::deserialize(deserializer)?)
            .map_err(|s| D::Error::invalid_value(Unexpected::Str(&s), &"a GraphQL name"))
    }
}

#[cfg(test)]
#[test]
fn test_valid_names() {
    assert!(Name::is_valid("valid_name"));
    assert!(Name::is_valid("numbers123_456_789abc"));
    assert!(Name::is_valid("MiXeD_CaSe"));
    assert!(Name::is_valid("_"));
    assert!(!Name::is_valid("invalid name"));
    assert!(!Name::is_valid("123and_text"));
}
