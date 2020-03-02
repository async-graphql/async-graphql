use async_graphql_derive::Enum;

#[Enum(internal)]
#[allow(non_camel_case_types)]
pub enum __TypeKind {
    #[item(desc = "Indicates this type is a scalar.")]
    SCALAR,

    #[item(desc = "Indicates this type is an object. `fields` and `interfaces` are valid fields.")]
    OBJECT,

    #[item(
        desc = "Indicates this type is an interface. `fields` and `possibleTypes` are valid fields."
    )]
    INTERFACE,

    #[item(desc = "Indicates this type is a union. `possibleTypes` is a valid field.")]
    UNION,

    #[item(desc = "Indicates this type is an enum. `enumValues` is a valid field.")]
    ENUM,

    #[item(desc = "Indicates this type is an input object. `inputFields` is a valid field.")]
    INPUT_OBJECT,

    #[item(desc = "Indicates this type is a list. `ofType` is a valid field.")]
    LIST,

    #[item(desc = "Indicates this type is a non-null. `ofType` is a valid field.")]
    NON_NULL,
}
