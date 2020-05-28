use crate::model::__Type;
use crate::registry;
use async_graphql_derive::Object;

pub struct __InputValue<'a> {
    pub registry: &'a registry::Registry,
    pub input_value: &'a registry::MetaInputValue,
}

/// Arguments provided to Fields or Directives and the input fields of an InputObject are represented as Input Values which describe their type and optionally a default value.
#[Object(internal)]
impl<'a> __InputValue<'a> {
    async fn name(&self) -> String {
        self.input_value.name.to_string()
    }

    async fn description(&self) -> Option<String> {
        self.input_value.description.map(|s| s.to_string())
    }

    #[field(name = "type")]
    async fn ty(&self) -> __Type<'a> {
        __Type::new(self.registry, &self.input_value.ty)
    }

    async fn default_value(&self) -> Option<String> {
        self.input_value.default_value.clone()
    }
}
