use crate::query_schema::__Type;
use crate::{Context, Result};
use async_graphql_derive::Object;

#[Object(
    internal,
    desc = "Object and Interface types are described by a list of Fields, each of which has a name, potentially a list of arguments, and a return type.",
    field(name = "name", type = "String", owned),
    field(name = "description", type = "Option<String>", owned),
    field(name = "type", resolver = "ty", type = "__Type", owned),
    field(name = "isDeprecated", type = "bool", owned),
    field(name = "deprecationReason", type = "Option<String>", owned)
)]
pub struct __Field {}

#[async_trait::async_trait]
#[allow()]
impl __FieldFields for __Field {
    async fn name(&self, _: &Context<'_>) -> Result<String> {
        todo!()
    }

    async fn description(&self, _: &Context<'_>) -> Result<Option<String>> {
        todo!()
    }

    async fn ty(&self, _: &Context<'_>) -> Result<__Type> {
        todo!()
    }

    async fn is_deprecated(&self, _: &Context<'_>) -> Result<bool> {
        todo!()
    }

    async fn deprecation_reason(&self, _: &Context<'_>) -> Result<Option<String>> {
        todo!()
    }
}
