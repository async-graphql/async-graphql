use crate::model::{__InputValue, __Type};
use crate::registry;
use async_graphql_derive::Object;

pub struct __Field<'a> {
    pub registry: &'a registry::Registry,
    pub field: &'a registry::Field,
}

#[Object(
    internal,
    desc = "Object and Interface types are described by a list of Fields, each of which has a name, potentially a list of arguments, and a return type."
)]
impl<'a> __Field<'a> {
    #[field]
    async fn name(&self) -> String {
        self.field.name.to_string()
    }

    #[field]
    async fn description(&self) -> Option<String> {
        self.field.description.map(|s| s.to_string())
    }

    #[field]
    async fn args(&self) -> Vec<__InputValue<'a>> {
        self.field
            .args
            .values()
            .map(|input_value| __InputValue {
                registry: self.registry,
                input_value,
            })
            .collect()
    }

    #[field(name = "type")]
    async fn ty(&self) -> __Type<'a> {
        __Type::new(self.registry, &self.field.ty)
    }

    #[field]
    async fn is_deprecated(&self) -> bool {
        self.field.deprecation.is_some()
    }

    #[field]
    async fn deprecation_reason(&self) -> Option<String> {
        self.field.deprecation.map(|s| s.to_string())
    }
}
