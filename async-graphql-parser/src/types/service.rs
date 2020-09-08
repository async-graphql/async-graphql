//! Service-related GraphQL types.

use super::*;

/// An GraphQL file or request string defining a GraphQL service.
///
/// [Reference](https://spec.graphql.org/June2018/#Document).
#[derive(Debug, Clone)]
pub struct ServiceDocument {
    /// The definitions of this document.
    pub definitions: Vec<TypeSystemDefinition>,
}

/// A definition concerning the type system of a GraphQL service.
///
/// [Reference](https://spec.graphql.org/June2018/#TypeSystemDefinition). This enum also covers
/// [extensions](https://spec.graphql.org/June2018/#TypeSystemExtension).
#[derive(Debug, Clone)]
pub enum TypeSystemDefinition {
    /// The definition of the schema of the service.
    Schema(Positioned<SchemaDefinition>),
    /// The definition of a type in the service.
    Type(Positioned<TypeDefinition>),
    /// The definition of a directive in the service.
    Directive(Positioned<DirectiveDefinition>),
}

/// The definition of the schema in a GraphQL service.
///
/// [Reference](https://spec.graphql.org/June2018/#SchemaDefinition). This also covers
/// [extensions](https://spec.graphql.org/June2018/#SchemaExtension).
#[derive(Debug, Clone)]
pub struct SchemaDefinition {
    /// Whether the schema is an extension of another schema.
    pub extend: bool,
    /// The directives of the schema definition.
    pub directives: Vec<Positioned<ConstDirective>>,
    /// The query root. This is always `Some` when the schema is not extended.
    pub query: Option<Positioned<Name>>,
    /// The mutation root, if present.
    pub mutation: Option<Positioned<Name>>,
    /// The subscription root, if present.
    pub subscription: Option<Positioned<Name>>,
}

/// The definition of a type in a GraphQL service.
///
/// [Reference](https://spec.graphql.org/June2018/#TypeDefinition). This also covers
/// [extensions](https://spec.graphql.org/June2018/#TypeExtension).
#[derive(Debug, Clone)]
pub struct TypeDefinition {
    /// Whether the type is an extension of another type.
    pub extend: bool,
    /// The description of the type, if present. This is never present on an extension type.
    pub description: Option<Positioned<String>>,
    /// The name of the type.
    pub name: Positioned<Name>,
    /// The directives of type definition.
    pub directives: Vec<Positioned<ConstDirective>>,
    /// Which kind of type is being defined; scalar, object, enum, etc.
    pub kind: TypeKind,
}

/// A kind of type; scalar, object, enum, etc.
#[derive(Debug, Clone)]
pub enum TypeKind {
    /// A scalar type.
    Scalar,
    /// An object type.
    Object(ObjectType),
    /// An interface type.
    Interface(InterfaceType),
    /// A union type.
    Union(UnionType),
    /// An enum type.
    Enum(EnumType),
    /// An input object type.
    InputObject(InputObjectType),
}

/// The definition of an object type.
///
/// [Reference](https://spec.graphql.org/June2018/#ObjectType).
#[derive(Debug, Clone)]
pub struct ObjectType {
    /// The interfaces implemented by the object.
    pub implements: Vec<Positioned<Name>>,
    /// The fields of the object type.
    pub fields: Vec<Positioned<FieldDefinition>>,
}

/// The definition of a field inside an object or interface.
///
/// [Reference](https://spec.graphql.org/June2018/#FieldDefinition).
#[derive(Debug, Clone)]
pub struct FieldDefinition {
    /// The description of the field.
    pub description: Option<Positioned<String>>,
    /// The name of the field.
    pub name: Positioned<Name>,
    /// The arguments of the field.
    pub arguments: Vec<Positioned<InputValueDefinition>>,
    /// The type of the field.
    pub ty: Positioned<Type>,
    /// The directives of the field.
    pub directives: Vec<Positioned<ConstDirective>>,
}

/// The definition of an interface type.
///
/// [Reference](https://spec.graphql.org/June2018/#InterfaceType).
#[derive(Debug, Clone)]
pub struct InterfaceType {
    /// The fields of the interface type.
    pub fields: Vec<Positioned<FieldDefinition>>,
}

/// The definition of a union type.
///
/// [Reference](https://spec.graphql.org/June2018/#UnionType).
#[derive(Debug, Clone)]
pub struct UnionType {
    /// The member types of the union.
    pub members: Vec<Positioned<Name>>,
}

/// The definition of an enum.
///
/// [Reference](https://spec.graphql.org/June2018/#EnumType).
#[derive(Debug, Clone)]
pub struct EnumType {
    /// The possible values of the enum.
    pub values: Vec<Positioned<EnumValueDefinition>>,
}

/// The definition of a value inside an enum.
///
/// [Reference](https://spec.graphql.org/June2018/#EnumValueDefinition).
#[derive(Debug, Clone)]
pub struct EnumValueDefinition {
    /// The description of the argument.
    pub description: Option<Positioned<String>>,
    /// The value name.
    pub value: Positioned<Name>,
    /// The directives of the enum value.
    pub directives: Vec<Positioned<ConstDirective>>,
}

/// The definition of an input object.
///
/// [Reference](https://spec.graphql.org/June2018/#InputObjectType).
#[derive(Debug, Clone)]
pub struct InputObjectType {
    /// The fields of the input object.
    pub fields: Vec<Positioned<InputValueDefinition>>,
}

/// The definition of an input value inside the arguments of a field.
///
/// [Reference](https://spec.graphql.org/June2018/#InputValueDefinition).
#[derive(Debug, Clone)]
pub struct InputValueDefinition {
    /// The description of the argument.
    pub description: Option<Positioned<String>>,
    /// The name of the argument.
    pub name: Positioned<Name>,
    /// The type of the argument.
    pub ty: Positioned<Type>,
    /// The default value of the argument, if there is one.
    pub default_value: Option<Positioned<ConstValue>>,
    /// The directives of the input value.
    pub directives: Vec<Positioned<ConstDirective>>,
}

/// The definition of a directive in a service.
///
/// [Reference](https://spec.graphql.org/June2018/#DirectiveDefinition).
#[derive(Debug, Clone)]
pub struct DirectiveDefinition {
    /// The description of the directive.
    pub description: Option<Positioned<String>>,
    /// The name of the directive.
    pub name: Positioned<Name>,
    /// The arguments of the directive.
    pub arguments: Vec<Positioned<InputValueDefinition>>,
    /// The locations the directive applies to.
    pub locations: Vec<Positioned<DirectiveLocation>>,
}

/// Where a directive can apply to.
///
/// [Reference](https://spec.graphql.org/June2018/#DirectiveLocation).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectiveLocation {
    /// A [query](enum.OperationType.html#variant.Query) [operation](struct.OperationDefinition.html).
    Query,
    /// A [mutation](enum.OperationType.html#variant.Mutation) [operation](struct.OperationDefinition.html).
    Mutation,
    /// A [subscription](enum.OperationType.html#variant.Subscription) [operation](struct.OperationDefinition.html).
    Subscription,
    /// A [field](struct.Field.html).
    Field,
    /// A [fragment definition](struct.FragmentDefinition.html).
    FragmentDefinition,
    /// A [fragment spread](struct.FragmentSpread.html).
    FragmentSpread,
    /// An [inline fragment](struct.InlineFragment.html).
    InlineFragment,
    /// A [schema](struct.Schema.html).
    Schema,
    /// A [scalar](enum.TypeKind.html#variant.Scalar).
    Scalar,
    /// An [object](struct.ObjectType.html).
    Object,
    /// A [field definition](struct.FieldDefinition.html).
    FieldDefinition,
    /// An [input value definition](struct.InputFieldDefinition.html) as the arguments of a field
    /// but not an input object.
    ArgumentDefinition,
    /// An [interface](struct.InterfaceType.html).
    Interface,
    /// A [union](struct.UnionType.html).
    Union,
    /// An [enum](struct.EnumType.html).
    Enum,
    /// A [value on an enum](struct.EnumValueDefinition.html).
    EnumValue,
    /// An [input object](struct.InputObjectType.html).
    InputObject,
    /// An [input value definition](struct.InputValueDefinition.html) on an input object but not a
    /// field.
    InputFieldDefinition,
}
