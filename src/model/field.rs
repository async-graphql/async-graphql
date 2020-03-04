use crate::model::{__InputValue, __Type};
use crate::{registry, Context, Result};
use async_graphql_derive::Object;

#[Object(
    internal,
    desc = "Object and Interface types are described by a list of Fields, each of which has a name, potentially a list of arguments, and a return type.",
    field(name = "name", type = "String", owned),
    field(name = "description", type = "Option<String>", owned),
    field(name = "args", type = "Vec<__InputValue>", owned),
    field(name = "type", resolver = "ty", type = "__Type", owned),
    field(name = "isDeprecated", type = "bool", owned),
    field(name = "deprecationReason", type = "Option<String>", owned)
)]
pub struct __Field<'a> {
    pub registry: &'a registry::Registry,
    pub field: &'a registry::Field,
}

#[async_trait::async_trait]
#[allow()]
impl<'a> __FieldFields for __Field<'a> {
    async fn name(&self, _: &Context<'_>) -> Result<String> {
        Ok(self.field.name.to_string())
    }

    async fn description(&self, _: &Context<'_>) -> Result<Option<String>> {
        Ok(self.field.description.map(|s| s.to_string()))
    }

    async fn args<'b>(&'b self, _: &Context<'_>) -> Result<Vec<__InputValue<'b>>> {
        Ok(self
            .field
            .args
            .iter()
            .map(|input_value| __InputValue {
                registry: self.registry,
                input_value,
            })
            .collect())
    }

    async fn ty<'b>(&'b self, _: &Context<'_>) -> Result<__Type<'b>> {
        Ok(__Type::new(self.registry, &self.field.ty))
    }

    async fn is_deprecated(&self, _: &Context<'_>) -> Result<bool> {
        Ok(self.field.deprecation.is_some())
    }

    async fn deprecation_reason(&self, _: &Context<'_>) -> Result<Option<String>> {
        Ok(self.field.deprecation.map(|s| s.to_string()))
    }
}
