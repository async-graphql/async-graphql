use async_graphql_derive::Enum;

#[Enum(
    internal,
    desc = "An enum describing what kind of type a given `__Type` is."
)]
pub enum __TypeKind {
    #[item(desc = "Indicates this type is a scalar.")]
    Scalar,

    #[item(desc = "Indicates this type is an object. `fields` and `interfaces` are valid fields.")]
    Object,

    #[item(
        desc = "Indicates this type is an interface. `fields` and `possibleTypes` are valid fields."
    )]
    Interface,

    #[item(desc = "Indicates this type is a union. `possibleTypes` is a valid field.")]
    Union,

    #[item(desc = "Indicates this type is an enum. `enumValues` is a valid field.")]
    Enum,

    #[item(desc = "Indicates this type is an input object. `inputFields` is a valid field.")]
    InputObject,

    #[item(desc = "Indicates this type is a list. `ofType` is a valid field.")]
    List,

    #[item(desc = "Indicates this type is a non-null. `ofType` is a valid field.")]
    NonNull,
}
