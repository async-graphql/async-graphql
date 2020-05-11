use async_graphql_derive::GqlEnum;

/// An enum describing what kind of type a given `__Type` is.
#[GqlEnum(internal)]
pub enum __TypeKind {
    /// Indicates this type is a scalar.
    Scalar,

    /// Indicates this type is an object. `fields` and `interfaces` are valid fields.
    Object,

    /// Indicates this type is an interface. `fields` and `possibleTypes` are valid fields.
    Interface,

    /// Indicates this type is a union. `possibleTypes` is a valid field.
    Union,

    /// Indicates this type is an enum. `enumValues` is a valid field.
    Enum,

    /// Indicates this type is an input object. `inputFields` is a valid field.
    InputObject,

    /// Indicates this type is a list. `ofType` is a valid field.
    List,

    /// Indicates this type is a non-null. `ofType` is a valid field.
    NonNull,
}
