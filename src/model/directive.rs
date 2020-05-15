use crate::model::__InputValue;
use crate::registry;
use async_graphql_derive::{Enum, Object};

/// A Directive can be adjacent to many parts of the GraphQL language, a __DirectiveLocation describes one such possible adjacencies.
#[Enum(internal)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum __DirectiveLocation {
    /// Location adjacent to a query operation.
    QUERY,

    /// Location adjacent to a mutation operation.
    MUTATION,

    /// Location adjacent to a subscription operation.
    SUBSCRIPTION,

    /// Location adjacent to a field.
    FIELD,

    /// Location adjacent to a fragment definition.
    FRAGMENT_DEFINITION,

    /// Location adjacent to a fragment spread.
    FRAGMENT_SPREAD,

    /// Location adjacent to an inline fragment.
    INLINE_FRAGMENT,

    /// Location adjacent to a variable definition.
    VARIABLE_DEFINITION,

    /// Location adjacent to a schema definition.
    SCHEMA,

    /// Location adjacent to a scalar definition.
    SCALAR,

    /// Location adjacent to an object type definition.
    OBJECT,

    /// Location adjacent to a field definition.
    FIELD_DEFINITION,

    /// Location adjacent to an argument definition.
    ARGUMENT_DEFINITION,

    /// Location adjacent to an interface definition.
    INTERFACE,

    /// Location adjacent to a union definition.
    UNION,

    /// Location adjacent to an enum definition.
    ENUM,

    /// Location adjacent to an enum value definition.
    ENUM_VALUE,

    /// Location adjacent to an input object type definition.
    INPUT_OBJECT,

    /// Location adjacent to an input object field definition.
    INPUT_FIELD_DEFINITION,
}

pub struct __Directive<'a> {
    pub registry: &'a registry::Registry,
    pub directive: &'a registry::MetaDirective,
}

/// A Directive provides a way to describe alternate runtime execution and type validation behavior in a GraphQL document.
//
// In some cases, you need to provide options to alter GraphQL's execution behavior in ways field arguments will not suffice, such as conditionally including or skipping a field. Directives provide this by describing additional information to the executor.
#[Object(internal)]
impl<'a> __Directive<'a> {
    async fn name(&self) -> String {
        self.directive.name.to_string()
    }

    async fn description(&self) -> Option<String> {
        self.directive.description.map(|s| s.to_string())
    }

    async fn locations(&self) -> &Vec<__DirectiveLocation> {
        &self.directive.locations
    }

    async fn args(&self) -> Vec<__InputValue<'a>> {
        self.directive
            .args
            .values()
            .map(|input_value| __InputValue {
                registry: self.registry,
                input_value,
            })
            .collect()
    }
}
