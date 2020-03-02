use crate::schema::__Type;
use crate::Value;
use async_graphql_derive::Object;

#[Object(internal)]
pub struct __InputValue {
    #[field(attr)]
    name: &'static str,

    #[field(attr)]
    description: Option<&'static str>,

    #[field(attr, name = "type")]
    ty: &'static __Type,

    #[field(attr, attr_type = "Value", name = "defaultValue")]
    default_value: String,
}
