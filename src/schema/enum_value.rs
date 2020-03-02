use crate::{ContextField, Result};
use async_graphql_derive::Object;

#[Object(
    internal,
    desc = "One possible value for a given Enum. Enum values are unique values, not a placeholder for a string or numeric value. However an Enum value is returned in a JSON response as a string.",
    field(name = "name", type = "String", owned),
    field(name = "description", type = "Option<String>", owned),
    field(name = "isDeprecated", type = "bool", owned),
    field(name = "deprecationReason", type = "Option<String>", owned)
)]
pub struct __EnumValue {}

#[async_trait::async_trait]
impl __EnumValueFields for __EnumValue {
    async fn name(&self, _: &ContextField<'_>) -> Result<String> {
        todo!()
    }

    async fn description(&self, _: &ContextField<'_>) -> Result<Option<String>> {
        todo!()
    }

    async fn is_deprecated(&self, _: &ContextField<'_>) -> Result<bool> {
        todo!()
    }

    async fn deprecation_reason(&self, _: &ContextField<'_>) -> Result<Option<String>> {
        todo!()
    }
}
