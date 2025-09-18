use std::collections::HashSet;

use crate::{Enum, Object, model::__InputValue, registry};

/// A Directive can be adjacent to many parts of the GraphQL language, a
/// __DirectiveLocation describes one such possible adjacencies.
#[derive(Debug, Enum, Copy, Clone, Eq, PartialEq, Default)]
#[graphql(internal, name = "__DirectiveLocation")]
#[allow(non_camel_case_types)]
pub enum __DirectiveLocation {
    /// Location adjacent to a query operation.
    #[default]
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

// Traits for compile time checking if location at which directive is called is
// supported by directives definition Would be nice to auto generate traits from
// variants of __DirectiveLocation
#[doc(hidden)]
#[allow(non_camel_case_types)]
pub mod location_traits {
    pub trait Directive_At_FIELD_DEFINITION {
        fn check() {}
    }

    pub trait Directive_At_OBJECT {
        fn check() {}
    }

    pub trait Directive_At_INPUT_FIELD_DEFINITION {
        fn check() {}
    }

    pub trait Directive_At_ARGUMENT_DEFINITION {
        fn check() {}
    }

    pub trait Directive_At_INPUT_OBJECT {
        fn check() {}
    }

    pub trait Directive_At_INTERFACE {
        fn check() {}
    }

    pub trait Directive_At_ENUM {
        fn check() {}
    }

    pub trait Directive_At_ENUM_VALUE {
        fn check() {}
    }
}

pub struct __Directive<'a> {
    pub registry: &'a registry::Registry,
    pub visible_types: &'a HashSet<&'a str>,
    pub directive: &'a registry::MetaDirective,
}

/// A Directive provides a way to describe alternate runtime execution and type
/// validation behavior in a GraphQL document.
///
/// In some cases, you need to provide options to alter GraphQL's execution
/// behavior in ways field arguments will not suffice, such as conditionally
/// including or skipping a field. Directives provide this by describing
/// additional information to the executor.
#[Object(internal, name = "__Directive")]
impl<'a> __Directive<'a> {
    #[inline]
    async fn name(&self) -> &str {
        &self.directive.name
    }

    #[inline]
    async fn description(&self) -> Option<&str> {
        self.directive.description.as_deref()
    }

    #[inline]
    async fn locations(&self) -> &Vec<__DirectiveLocation> {
        &self.directive.locations
    }

    async fn args(
        &self,
        #[graphql(default = false)] include_deprecated: bool,
    ) -> Vec<__InputValue<'a>> {
        self.directive
            .args
            .values()
            .filter(|input_value| include_deprecated || !input_value.deprecation.is_deprecated())
            .map(|input_value| __InputValue {
                registry: self.registry,
                visible_types: self.visible_types,
                input_value,
            })
            .collect()
    }

    #[inline]
    async fn is_repeatable(&self) -> bool {
        self.directive.is_repeatable
    }
}
