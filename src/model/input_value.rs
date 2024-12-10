use std::collections::HashSet;

use crate::{model::__Type, registry, Object};

pub struct __InputValue<'a> {
    pub registry: &'a registry::Registry,
    pub visible_types: &'a HashSet<&'a str>,
    pub input_value: &'a registry::MetaInputValue,
}

/// Arguments provided to Fields or Directives and the input fields of an
/// InputObject are represented as Input Values which describe their type and
/// optionally a default value.
#[Object(internal, name = "__InputValue")]
impl<'a> __InputValue<'a> {
    #[inline]
    async fn name(&self) -> &str {
        &self.input_value.name
    }

    #[inline]
    async fn description(&self) -> Option<&str> {
        self.input_value.description.as_deref()
    }

    #[graphql(name = "type")]
    #[inline]
    async fn ty(&self) -> __Type<'a> {
        __Type::new(self.registry, self.visible_types, &self.input_value.ty)
    }

    #[inline]
    async fn default_value(&self) -> Option<&str> {
        self.input_value.default_value.as_deref()
    }

    #[inline]
    async fn is_deprecated(&self) -> bool {
        self.input_value.deprecation.is_deprecated()
    }

    #[inline]
    async fn deprecation_reason(&self) -> Option<&str> {
        self.input_value.deprecation.reason()
    }
}
