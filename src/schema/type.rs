use crate::schema::{__Field, __InputValue, __TypeKind};
use async_graphql_derive::Object;

#[Object(internal)]
pub struct __Type {
    #[field(attr)]
    kind: __TypeKind,

    #[field(attr)]
    name: Option<&'static str>,

    #[field(attr)]
    description: Option<&'static str>,

    #[field(attr, arg(name = "includeDeprecated", type = "bool"))]
    fields: Option<&'static [__Field]>,

    #[field(attr)]
    interfaces: Option<&'static [__Type]>,

    #[field(attr, name = "possibleTypes")]
    possible_types: Option<&'static [__Type]>,

    #[field(
        attr,
        name = "enumValues",
        arg(name = "includeDeprecated", type = "bool")
    )]
    enum_values: Option<&'static [__Type]>,

    #[field(attr, name = "inputFields")]
    input_fields: Option<&'static [__InputValue]>,

    #[field(attr, name = "ofType")]
    of_type: Option<&'static __Type>,
}
