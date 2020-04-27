use crate::{registry, Context};
use async_graphql_derive::Object;

pub struct __EnumValue<'a> {
    pub registry: &'a registry::Registry,
    pub value: &'a registry::EnumValue,
}

#[Object(
    internal,
    desc = "One possible value for a given Enum. Enum values are unique values, not a placeholder for a string or numeric value. However an Enum value is returned in a JSON response as a string."
)]
impl<'a> __EnumValue<'a> {
    async fn name(&self, _: &Context<'_>) -> String {
        self.value.name.to_string()
    }

    async fn description(&self, _: &Context<'_>) -> Option<String> {
        self.value.description.map(|s| s.to_string())
    }

    async fn is_deprecated(&self, _: &Context<'_>) -> bool {
        self.value.deprecation.is_some()
    }

    async fn deprecation_reason(&self, _: &Context<'_>) -> Option<String> {
        self.value.deprecation.map(|s| s.to_string())
    }
}
