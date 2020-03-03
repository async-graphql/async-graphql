use crate::{registry, Context, Result};
use async_graphql_derive::Object;

#[Object(
    internal,
    desc = "One possible value for a given Enum. Enum values are unique values, not a placeholder for a string or numeric value. However an Enum value is returned in a JSON response as a string.",
    field(name = "name", type = "String", owned),
    field(name = "description", type = "Option<String>", owned),
    field(name = "isDeprecated", type = "bool", owned),
    field(name = "deprecationReason", type = "Option<String>", owned)
)]
pub struct __EnumValue<'a> {
    pub registry: &'a registry::Registry,
    pub value: &'a registry::EnumValue,
}

#[async_trait::async_trait]
impl<'a> __EnumValueFields for __EnumValue<'a> {
    async fn name(&self, _: &Context<'_>) -> Result<String> {
        Ok(self.value.name.to_string())
    }

    async fn description(&self, _: &Context<'_>) -> Result<Option<String>> {
        Ok(self.value.description.map(|s| s.to_string()))
    }

    async fn is_deprecated(&self, _: &Context<'_>) -> Result<bool> {
        Ok(self.value.deprecation.is_some())
    }

    async fn deprecation_reason(&self, _: &Context<'_>) -> Result<Option<String>> {
        Ok(self.value.deprecation.map(|s| s.to_string()))
    }
}
