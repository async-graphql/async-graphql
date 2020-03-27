use crate::model::__InputValue;
use crate::registry;
use async_graphql_derive::{Enum, Object};

#[Enum(
    internal,
    desc = "A Directive can be adjacent to many parts of the GraphQL language, a __DirectiveLocation describes one such possible adjacencies."
)]
#[allow(non_camel_case_types)]
pub enum __DirectiveLocation {
    #[item(desc = "Location adjacent to a query operation.")]
    QUERY,

    #[item(desc = "Location adjacent to a mutation operation.")]
    MUTATION,

    #[item(desc = "Location adjacent to a subscription operation.")]
    SUBSCRIPTION,

    #[item(desc = "Location adjacent to a field.")]
    FIELD,

    #[item(desc = "Location adjacent to a fragment definition.")]
    FRAGMENT_DEFINITION,

    #[item(desc = "Location adjacent to a fragment spread.")]
    FRAGMENT_SPREAD,

    #[item(desc = "Location adjacent to an inline fragment.")]
    INLINE_FRAGMENT,

    #[item(desc = "Location adjacent to a variable definition.")]
    VARIABLE_DEFINITION,

    #[item(desc = "Location adjacent to a schema definition.")]
    SCHEMA,

    #[item(desc = "Location adjacent to a scalar definition.")]
    SCALAR,

    #[item(desc = "Location adjacent to an object type definition.")]
    OBJECT,

    #[item(desc = "Location adjacent to a field definition.")]
    FIELD_DEFINITION,

    #[item(desc = "Location adjacent to an argument definition.")]
    ARGUMENT_DEFINITION,

    #[item(desc = "Location adjacent to an interface definition.")]
    INTERFACE,

    #[item(desc = "Location adjacent to a union definition.")]
    UNION,

    #[item(desc = "Location adjacent to an enum definition.")]
    ENUM,

    #[item(desc = "Location adjacent to an enum value definition.")]
    ENUM_VALUE,

    #[item(desc = "Location adjacent to an input object type definition.")]
    INPUT_OBJECT,

    #[item(desc = "Location adjacent to an input object field definition.")]
    INPUT_FIELD_DEFINITION,
}

pub struct __Directive<'a> {
    pub registry: &'a registry::Registry,
    pub directive: &'a registry::Directive,
}

#[Object(
    internal,
    desc = r#"A Directive provides a way to describe alternate runtime execution and type validation behavior in a GraphQL document.

In some cases, you need to provide options to alter GraphQL's execution behavior in ways field arguments will not suffice, such as conditionally including or skipping a field. Directives provide this by describing additional information to the executor."#
)]
impl<'a> __Directive<'a> {
    #[field]
    fn name(&self) -> String {
        self.directive.name.to_string()
    }

    #[field]
    fn description(&self) -> Option<String> {
        self.directive.description.map(|s| s.to_string())
    }

    #[field]
    fn locations(&self) -> &Vec<__DirectiveLocation> {
        &self.directive.locations
    }

    #[field]
    fn args(&self) -> Vec<__InputValue<'a>> {
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
