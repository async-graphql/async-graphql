use crate::schema::{__InputValue, __Type};
use async_graphql_derive::Object;

#[Object(internal, auto_impl)]
pub struct __Field {
    #[field(attr)]
    name: &'static str,

    #[field(attr)]
    description: Option<&'static str>,

    #[field(attr)]
    args: &'static [__InputValue],

    #[field(attr, name = "type")]
    ty: &'static __Type,

    #[field(attr, name = "isDeprecated")]
    is_deprecated: bool,

    #[field(attr, name = "deprecationReason")]
    deprecation_reason: Option<String>,
}
