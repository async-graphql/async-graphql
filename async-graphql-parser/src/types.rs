//! GraphQL types.

use crate::pos::{Pos, Positioned};
use serde::{Serialize, Serializer};
use std::collections::{BTreeMap, HashMap};
use std::fmt::{self, Formatter, Write};
use std::fs::File;

/// A complete GraphQL file or request string.
///
/// [Reference](https://spec.graphql.org/June2018/#Document).
#[derive(Debug, Clone)]
pub struct Document {
    /// The definitions of the document.
    pub definitions: Vec<Definition>,
}

impl Document {
    /// Convert the document into an [`ExecutableDocument`](struct.ExecutableDocument). Will return
    /// `None` if there is no operation in the document.
    ///
    /// The `operation_name` parameter, if set, makes sure that the main operation of the document,
    /// if named, will have that name.
    #[must_use]
    pub fn into_executable(self, operation_name: Option<&str>) -> Option<ExecutableDocument> {
        let mut operation = None;
        let mut fragments = HashMap::new();

        for definition in self.definitions {
            match definition {
                Definition::Operation(op)
                    if operation_name
                        .zip(op.node.name.as_ref())
                        .map_or(false, |(required_name, op_name)| {
                            required_name != op_name.node
                        }) =>
                {
                    ()
                }
                Definition::Operation(op) => {
                    operation.get_or_insert(op);
                }
                Definition::Fragment(fragment) => {
                    fragments.insert(fragment.node.name.node.clone(), fragment);
                }
            }
        }
        operation.map(|operation| ExecutableDocument {
            operation,
            fragments,
        })
    }
}

/// An executable document. This is a [`Document`](struct.Document.html) with at least one
/// operation, and any number of fragments.
#[derive(Debug, Clone)]
pub struct ExecutableDocument {
    /// The main operation of the document.
    pub operation: Positioned<OperationDefinition>,
    /// The fragments of the document.
    pub fragments: HashMap<String, Positioned<FragmentDefinition>>,
}

/// An executable definition in a query; a query, mutation, subscription or fragment definition.
///
/// [Reference](https://spec.graphql.org/June2018/#ExecutableDefinition).
#[derive(Debug, Clone)]
pub enum Definition {
    /// The definition of an operation.
    Operation(Positioned<OperationDefinition>),
    /// The definition of a fragment.
    Fragment(Positioned<FragmentDefinition>),
}

impl Definition {
    /// Get the position of the definition.
    #[must_use]
    pub fn pos(&self) -> Pos {
        match self {
            Self::Operation(op) => op.pos,
            Self::Fragment(frag) => frag.pos,
        }
    }

    /// Get a reference to the directives of the definition.
    #[must_use]
    pub fn directives(&self) -> &Vec<Positioned<Directive>> {
        match self {
            Self::Operation(op) => &op.node.directives,
            Self::Fragment(frag) => &frag.node.directives,
        }
    }
    /// Get a mutable reference to the directives of the definition.
    #[must_use]
    pub fn directives_mut(&mut self) -> &mut Vec<Positioned<Directive>> {
        match self {
            Self::Operation(op) => &mut op.node.directives,
            Self::Fragment(frag) => &mut frag.node.directives,
        }
    }
}

/// A GraphQL operation, such as `mutation($content:String!) { makePost(content: $content) { id } }`.
///
/// [Reference](https://spec.graphql.org/June2018/#OperationDefinition).
#[derive(Debug, Clone)]
pub struct OperationDefinition {
    /// The type of operation.
    pub ty: OperationType,
    /// The name of the operation.
    pub name: Option<Positioned<String>>,
    /// The variable definitions.
    pub variable_definitions: Vec<Positioned<VariableDefinition>>,
    /// The operation's directives.
    pub directives: Vec<Positioned<Directive>>,
    /// The operation's selection set.
    pub selection_set: Positioned<SelectionSet>,
}

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

impl fmt::Display for OperationType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Query => "query",
            Self::Mutation => "mutation",
            Self::Subscription => "subscription",
        })
    }
}

/// A variable definition inside a list of variable definitions, for example `$name:String!`.
///
/// [Reference](https://spec.graphql.org/June2018/#VariableDefinition).
#[derive(Debug, Clone)]
pub struct VariableDefinition {
    /// The name of the variable, without the preceding `$`.
    pub name: Positioned<String>,
    /// The type of the variable.
    pub var_type: Positioned<Type>,
    /// The optional default value of the variable.
    pub default_value: Option<Positioned<Value>>,
}

impl VariableDefinition {
    /// Get the default value of the variable; this is `default_value` if it is present,
    /// `Value::Null` if it is nullable and `None` otherwise.
    #[must_use]
    pub fn default_value(&self) -> Option<&Value> {
        self.default_value
            .as_ref()
            .map(|value| &value.node)
            .or_else(|| {
                if self.var_type.node.nullable {
                    Some(&Value::Null)
                } else {
                    None
                }
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
    pub fn new(ty: &str) -> Self {
        let (nullable, ty) = if let Some(rest) = ty.strip_suffix('!') {
            (false, rest)
        } else {
            (true, ty)
        };

        Self {
            base: if let Some(ty) = ty.strip_prefix('[').and_then(|ty| ty.strip_suffix(']')) {
                BaseType::List(Box::new(Self::new(ty)))
            } else {
                BaseType::Named(ty.to_owned())
            },
            nullable,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
    Named(String),
    /// A list type, such as `[String]`.
    List(Box<Type>),
}

impl fmt::Display for BaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Named(name) => f.write_str(name),
            Self::List(ty) => write!(f, "[{}]", ty),
        }
    }
}

/// A GraphQL value, for example `1`, `$name` or `"Hello World!"`.
///
/// [Reference](https://spec.graphql.org/June2018/#Value).
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Value {
    /// `null`.
    Null,
    /// A variable, without the `$`.
    #[serde(serialize_with = "serialize_variable")]
    Variable(String),
    /// A number.
    Number(serde_json::Number),
    /// A string.
    String(String),
    /// A boolean.
    Boolean(bool),
    /// An enum. These are typically in `SCREAMING_SNAKE_CASE`.
    Enum(String),
    /// A list of values.
    List(Vec<Value>),
    /// An object. This is a map of keys to values.
    Object(BTreeMap<String, Value>),
    /// An uploaded file.
    #[serde(serialize_with = "serialize_upload")]
    Upload(UploadValue),
}
fn serialize_variable<S: Serializer>(name: &str, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&format!("${}", name))
}
fn serialize_upload<S: Serializer>(_: &UploadValue, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_unit()
}

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Variable(name) => write!(f, "${}", name),
            Value::Number(num) => write!(f, "{}", *num),
            Value::String(val) => write_quoted(val, f),
            Value::Boolean(true) => f.write_str("true"),
            Value::Boolean(false) => f.write_str("false"),
            Value::Null | Value::Upload(_) => f.write_str("null"),
            Value::Enum(name) => f.write_str(name),
            Value::List(items) => {
                f.write_char('[')?;
                for (i, item) in items.iter().enumerate() {
                    if i != 0 {
                        f.write_str(", ")?;
                    }
                    item.fmt(f)?;
                }
                f.write_char(']')
            }
            Value::Object(items) => {
                f.write_str("{")?;
                for (i, (name, value)) in items.iter().enumerate() {
                    write!(f, "{}{}: {}", if i == 0 { "" } else { ", " }, name, value)?;
                }
                f.write_str("}")
            }
        }
    }
}

fn write_quoted(s: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

impl From<Value> for serde_json::Value {
    fn from(value: Value) -> Self {
        match value {
            Value::Null | Value::Upload(_) => serde_json::Value::Null,
            Value::Variable(name) => name.into(),
            Value::Number(n) => serde_json::Value::Number(n),
            Value::String(s) => s.into(),
            Value::Boolean(v) => v.into(),
            Value::Enum(e) => e.into(),
            Value::List(values) => values
                .into_iter()
                .map(Into::into)
                .collect::<Vec<serde_json::Value>>()
                .into(),
            Value::Object(obj) => serde_json::Value::Object(
                obj.into_iter()
                    .map(|(name, value)| (name, value.into()))
                    .collect(),
            ),
        }
    }
}

impl From<serde_json::Value> for Value {
    fn from(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(n) => Value::Boolean(n),
            serde_json::Value::Number(n) => Value::Number(n),
            serde_json::Value::String(s) => Value::String(s),
            serde_json::Value::Array(ls) => Value::List(ls.into_iter().map(Into::into).collect()),
            serde_json::Value::Object(obj) => Value::Object(
                obj.into_iter()
                    .map(|(name, value)| (name, value.into()))
                    .collect(),
            ),
        }
    }
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

/// A set of fields to be selected, for example `{ name age }`.
///
/// [Reference](https://spec.graphql.org/June2018/#SelectionSet).
#[derive(Debug, Default, Clone)]
pub struct SelectionSet {
    /// The fields to be selected.
    pub items: Vec<Positioned<Selection>>,
}

/// A part of an object to be selected; a single field, a fragment spread or an inline fragment.
///
/// [Reference](https://spec.graphql.org/June2018/#Selection).
#[derive(Debug, Clone)]
pub enum Selection {
    /// Select a single field, such as `name` or `weightKilos: weight(unit: KILOGRAMS)`.
    Field(Positioned<Field>),
    /// Select using a fragment.
    FragmentSpread(Positioned<FragmentSpread>),
    /// Select using an inline fragment.
    InlineFragment(Positioned<InlineFragment>),
}

impl Selection {
    /// Get a reference to the directives of the selection.
    #[must_use]
    pub fn directives(&self) -> &Vec<Positioned<Directive>> {
        match self {
            Self::Field(field) => &field.node.directives,
            Self::FragmentSpread(spread) => &spread.node.directives,
            Self::InlineFragment(fragment) => &fragment.node.directives,
        }
    }
    /// Get a mutable reference to the directives of the selection.
    #[must_use]
    pub fn directives_mut(&mut self) -> &mut Vec<Positioned<Directive>> {
        match self {
            Self::Field(field) => &mut field.node.directives,
            Self::FragmentSpread(spread) => &mut spread.node.directives,
            Self::InlineFragment(fragment) => &mut fragment.node.directives,
        }
    }
}

/// A field being selected on an object, such as `name` or `weightKilos: weight(unit: KILOGRAMS)`.
///
/// [Reference](https://spec.graphql.org/June2018/#Field).
#[derive(Debug, Clone)]
pub struct Field {
    /// The optional field alias.
    pub alias: Option<Positioned<String>>,
    /// The name of the field.
    pub name: Positioned<String>,
    /// The arguments to the field, empty if no arguments are provided.
    pub arguments: Vec<(Positioned<String>, Positioned<Value>)>,
    /// The directives in the field selector.
    pub directives: Vec<Positioned<Directive>>,
    /// The subfields being selected in this field, if it is an object. Empty if no fields are
    /// being selected.
    pub selection_set: Positioned<SelectionSet>,
}

impl Field {
    /// Get the response key of the field. This is the alias if present and the name otherwise.
    #[must_use]
    pub fn response_key(&self) -> &Positioned<String> {
        self.alias.as_ref().unwrap_or(&self.name)
    }

    /// Get the value of the argument with the specified name.
    #[must_use]
    pub fn get_argument(&self, name: &str) -> Option<&Positioned<Value>> {
        self.arguments
            .iter()
            .find(|item| item.0.node == name)
            .map(|item| &item.1)
    }
}

/// A fragment selector, such as `... userFields`.
///
/// [Reference](https://spec.graphql.org/June2018/#FragmentSpread).
#[derive(Debug, Clone)]
pub struct FragmentSpread {
    /// The name of the fragment being selected.
    pub fragment_name: Positioned<String>,
    /// The directives in the fragment selector.
    pub directives: Vec<Positioned<Directive>>,
}

/// An inline fragment selector, such as `... on User { name }`.
///
/// [Reference](https://spec.graphql.org/June2018/#InlineFragment).
#[derive(Debug, Clone)]
pub struct InlineFragment {
    /// The type condition.
    pub type_condition: Option<Positioned<TypeCondition>>,
    /// The directives in the inline fragment.
    pub directives: Vec<Positioned<Directive>>,
    /// The selected fields of the fragment.
    pub selection_set: Positioned<SelectionSet>,
}

/// The definition of a fragment, such as `fragment userFields on User { name age }`.
///
/// [Reference](https://spec.graphql.org/June2018/#FragmentDefinition).
#[derive(Debug, Clone)]
pub struct FragmentDefinition {
    /// The name of the fragment. Any name is allowed except `on`.
    pub name: Positioned<String>,
    /// The type this fragment operates on.
    pub type_condition: Positioned<TypeCondition>,
    /// Directives in the fragment.
    pub directives: Vec<Positioned<Directive>>,
    /// The fragment's selection set.
    pub selection_set: Positioned<SelectionSet>,
}

/// A type a fragment can apply to (`on` followed by the type).
///
/// [Reference](https://spec.graphql.org/June2018/#TypeCondition).
#[derive(Debug, Clone)]
pub struct TypeCondition {
    /// The type this fragment applies to.
    pub on: Positioned<String>,
}

/// A GraphQL directive, such as `@deprecated(reason: "Use the other field")`.
///
/// [Reference](https://spec.graphql.org/June2018/#Directive).
#[derive(Debug, Clone)]
pub struct Directive {
    /// The name of the directive.
    pub name: Positioned<String>,
    /// The arguments to the directive.
    pub arguments: Vec<(Positioned<String>, Positioned<Value>)>,
}

impl Directive {
    /// Get the argument with the given name.
    #[must_use]
    pub fn get_argument(&self, name: &str) -> Option<&Positioned<Value>> {
        self.arguments
            .iter()
            .find(|item| item.0.node == name)
            .map(|item| &item.1)
    }
}
