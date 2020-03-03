use crate::model::__Type;
use crate::{registry, Context, Result};
use async_graphql_derive::Object;

#[Object(
    internal,
    desc = "Arguments provided to Fields or Directives and the input fields of an InputObject are represented as Input Values which describe their type and optionally a default value.",
    field(name = "name", type = "String", owned),
    field(name = "description", type = "Option<String>", owned),
    field(name = "type", resolver = "ty", type = "__Type", owned),
    field(name = "defaultValue", type = "Option<String>", owned)
)]
pub struct __InputValue<'a> {
    pub registry: &'a registry::Registry,
    pub input_value: &'a registry::InputValue,
}

#[async_trait::async_trait]
impl<'a> __InputValueFields for __InputValue<'a> {
    async fn name(&self, _: &Context<'_>) -> Result<String> {
        Ok(self.input_value.name.to_string())
    }

    async fn description(&self, _: &Context<'_>) -> Result<Option<String>> {
        Ok(self.input_value.description.map(|s| s.to_string()))
    }

    async fn ty<'b>(&'b self, _: &Context<'_>) -> Result<__Type<'b>> {
        Ok(__Type {
            registry: self.registry,
            ty: &self.registry[&self.input_value.ty],
        })
    }

    async fn default_value(&self, _: &Context<'_>) -> Result<Option<String>> {
        Ok(self.input_value.default_value.map(|s| s.to_string()))
    }
}
