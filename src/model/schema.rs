use crate::model::{__Directive, __Type};
use crate::registry;
use async_graphql_derive::Object;
use itertools::Itertools;

pub struct __Schema<'a> {
    pub registry: &'a registry::Registry,
}

/// A GraphQL Schema defines the capabilities of a GraphQL server. It exposes all available types and directives on the server, as well as the entry points for query, mutation, and subscription operations.
#[Object(internal)]
impl<'a> __Schema<'a> {
    /// A list of all types supported by this server.
    async fn types(&self) -> Vec<__Type<'a>> {
        let mut types = self
            .registry
            .types
            .values()
            .map(|ty| (ty.name(), __Type::new_simple(self.registry, ty)))
            .collect_vec();
        types.sort_by(|a, b| a.0.cmp(b.0));
        types.into_iter().map(|(_, ty)| ty).collect()
    }

    /// The type that query operations will be rooted at.
    async fn query_type(&self) -> __Type<'a> {
        __Type::new_simple(
            self.registry,
            &self.registry.types[&self.registry.query_type],
        )
    }

    /// If this server supports mutation, the type that mutation operations will be rooted at.
    async fn mutation_type(&self) -> Option<__Type<'a>> {
        if let Some(ty) = &self.registry.mutation_type {
            Some(__Type::new_simple(self.registry, &self.registry.types[ty]))
        } else {
            None
        }
    }

    /// If this server support subscription, the type that subscription operations will be rooted at.
    async fn subscription_type(&self) -> Option<__Type<'a>> {
        if let Some(ty) = &self.registry.subscription_type {
            Some(__Type::new_simple(self.registry, &self.registry.types[ty]))
        } else {
            None
        }
    }

    /// A list of all directives supported by this server.
    async fn directives(&self) -> Vec<__Directive<'a>> {
        let mut directives = self
            .registry
            .directives
            .values()
            .map(|directive| __Directive {
                registry: &self.registry,
                directive,
            })
            .collect_vec();
        directives.sort_by(|a, b| a.directive.name.cmp(b.directive.name));
        directives
    }
}
