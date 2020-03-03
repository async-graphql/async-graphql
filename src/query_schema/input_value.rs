use crate::query_schema::__Type;
use crate::{Context, Result};
use async_graphql_derive::Object;

#[Object(
    internal,
    desc = "Arguments provided to Fields or Directives and the input fields of an InputObject are represented as Input Values which describe their type and optionally a default value.",
    field(name = "name", type = "String", owned),
    field(name = "description", type = "Option<String>", owned),
    field(name = "type", resolver = "ty", type = "__Type", owned),
    field(name = "defaultValue", type = "String", owned)
)]
pub struct __InputValue {}

#[async_trait::async_trait]
impl __InputValueFields for __InputValue {
    async fn name(&self, _: &Context<'_>) -> Result<String> {
        todo!()
    }

    async fn description(&self, _: &Context<'_>) -> Result<Option<String>> {
        todo!()
    }

    async fn ty(&self, _: &Context<'_>) -> Result<__Type> {
        todo!()
    }

    async fn default_value(&self, _: &Context<'_>) -> Result<String> {
        todo!()
    }
}
