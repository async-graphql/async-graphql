use crate::pos::Positioned;
use crate::ParsedValue;

#[derive(Debug, PartialEq)]
pub enum Type {
    Named(String),
    List(Box<Type>),
    NonNull(Box<Type>),
}

#[derive(Debug)]
pub struct Document {
    pub definitions: Vec<Positioned<Definition>>,
}

#[derive(Debug)]
pub enum Definition {
    SchemaDefinition(Positioned<SchemaDefinition>),
    TypeDefinition {
        extend: bool,
        description: Positioned<String>,
        definition: Positioned<TypeDefinition>,
    },
    DirectiveDefinition(Positioned<DirectiveDefinition>),
}

#[derive(Debug)]
pub struct SchemaDefinition {
    pub directives: Vec<Positioned<Directive>>,
    pub query: Option<Positioned<String>>,
    pub mutation: Option<Positioned<String>>,
    pub subscription: Option<Positioned<String>>,
}

#[derive(Debug)]
pub enum TypeDefinition {
    Scalar(Positioned<ScalarType>),
    Object(Positioned<ObjectType>),
    Interface(Positioned<InterfaceType>),
    Union(Positioned<UnionType>),
    Enum(Positioned<EnumType>),
    InputObject(Positioned<InputObjectType>),
}

#[derive(Debug)]
pub struct ScalarType {
    pub name: Positioned<String>,
    pub directives: Vec<Positioned<Directive>>,
}

#[derive(Debug)]
pub struct ObjectType {
    pub name: Positioned<String>,
    pub implements_interfaces: Vec<Positioned<String>>,
    pub directives: Vec<Positioned<Directive>>,
    pub fields: Vec<Positioned<Field>>,
}

#[derive(Debug)]
pub struct Field {
    pub description: Option<Positioned<String>>,
    pub name: Positioned<String>,
    pub arguments: Vec<Positioned<InputValue>>,
    pub ty: Positioned<Type>,
    pub directives: Vec<Positioned<Directive>>,
}

#[derive(Debug)]
pub struct InputValue {
    pub description: Option<Positioned<String>>,
    pub name: Positioned<String>,
    pub ty: Positioned<Type>,
    pub default_value: Option<ParsedValue>,
    pub directives: Vec<Positioned<Directive>>,
}

#[derive(Debug)]
pub struct InterfaceType {
    pub name: Positioned<String>,
    pub directives: Vec<Positioned<Directive>>,
    pub fields: Vec<Positioned<Field>>,
}

#[derive(Debug)]
pub struct UnionType {
    pub name: Positioned<String>,
    pub directives: Vec<Positioned<Directive>>,
    pub types: Vec<Positioned<String>>,
}

#[derive(Debug)]
pub struct EnumType {
    pub name: Positioned<String>,
    pub directives: Vec<Positioned<Directive>>,
    pub values: Vec<Positioned<EnumValue>>,
}

#[derive(Debug)]
pub struct EnumValue {
    pub description: Option<Positioned<String>>,
    pub name: Positioned<String>,
    pub directives: Vec<Positioned<Directive>>,
}

#[derive(Debug)]
pub struct InputObjectType {
    pub name: Positioned<String>,
    pub directives: Vec<Positioned<Directive>>,
    pub fields: Vec<Positioned<InputValue>>,
}

#[derive(Debug)]
pub enum DirectiveLocation {
    // executable
    Query,
    Mutation,
    Subscription,
    Field,
    FragmentDefinition,
    FragmentSpread,
    InlineFragment,

    // type_system
    Schema,
    Scalar,
    Object,
    FieldDefinition,
    ArgumentDefinition,
    Interface,
    Union,
    Enum,
    EnumValue,
    InputObject,
    InputFieldDefinition,
}

#[derive(Debug)]
pub struct DirectiveDefinition {
    pub description: Option<Positioned<String>>,
    pub name: Positioned<String>,
    pub arguments: Vec<Positioned<InputValue>>,
    pub locations: Vec<Positioned<DirectiveLocation>>,
}

#[derive(Debug)]
pub struct Directive {
    pub name: Positioned<String>,
    pub arguments: Vec<(Positioned<String>, Positioned<ParsedValue>)>,
}
