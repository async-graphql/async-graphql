use crate::model::__Type;
use crate::registry;
use async_graphql_derive::Object;

pub struct __InputValue<'a> {
    pub registry: &'a registry::Registry,
    pub input_value: &'a registry::InputValue,
}

#[Object(
    internal,
    desc = "Arguments provided to Fields or Directives and the input fields of an InputObject are represented as Input Values which describe their type and optionally a default value."
)]
impl<'a> __InputValue<'a> {
    #[field]
    fn name(&self) -> String {
        self.input_value.name.to_string()
    }

    #[field]
    fn description(&self) -> Option<String> {
        self.input_value.description.map(|s| s.to_string())
    }

    #[field(name = "type")]
    fn ty(&self) -> __Type<'a> {
        __Type::new(self.registry, &self.input_value.ty)
    }

    #[field]
    fn default_value(&self) -> Option<String> {
        self.input_value.default_value.map(|s| s.to_string())
    }
}
