use crate::model::{__Directive, __Type};
use crate::registry;
use crate::{Context, Result};
use async_graphql_derive::Object;

#[Object(
    internal,
    desc = "A GraphQL Schema defines the capabilities of a GraphQL server. It exposes all available types and directives on the server, as well as the entry points for query, mutation, and subscription operations.",
    field(
        name = "types",
        desc = "A list of all types supported by this server.",
        type = "Vec<__Type>",
        owned
    ),
    field(
        name = "queryType",
        desc = "The type that query operations will be rooted at.",
        type = "__Type",
        owned
    ),
    field(
        name = "mutationType",
        desc = "If this server supports mutation, the type that mutation operations will be rooted at.",
        type = "Option<__Type>",
        owned
    ),
    field(
        name = "subscriptionType",
        desc = "If this server support subscription, the type that subscription operations will be rooted at.",
        type = "Option<__Type>",
        owned
    ),
    field(
        name = "directives",
        desc = "A list of all directives supported by this server.",
        type = "Vec<__Directive>",
        owned
    )
)]
pub struct __Schema<'a> {
    pub registry: &'a registry::Registry,
    pub query_type: &'a str,
    pub mutation_type: &'a str,
}

#[async_trait::async_trait]
impl<'a> __SchemaFields for __Schema<'a> {
    async fn types<'b>(&'b self, _: &Context<'_>) -> Result<Vec<__Type<'b>>> {
        Ok(self
            .registry
            .types
            .values()
            .map(|ty| __Type::new_simple(self.registry, ty))
            .collect())
    }

    async fn query_type<'b>(&'b self, _: &Context<'_>) -> Result<__Type<'b>> {
        Ok(__Type::new_simple(
            self.registry,
            &self.registry.types[self.query_type],
        ))
    }

    async fn mutation_type<'b>(&'b self, _: &Context<'_>) -> Result<Option<__Type<'b>>> {
        Ok(Some(__Type::new_simple(
            self.registry,
            &self.registry.types[self.mutation_type],
        )))
    }

    async fn subscription_type<'b>(&'b self, _: &Context<'_>) -> Result<Option<__Type<'b>>> {
        Ok(None)
    }

    async fn directives<'b>(&'b self, _: &Context<'_>) -> Result<Vec<__Directive<'b>>> {
        Ok(self
            .registry
            .directives
            .iter()
            .map(|directive| __Directive {
                registry: &self.registry,
                directive,
            })
            .collect())
    }
}
