use std::collections::HashSet;

use crate::{
    model::{__InputValue, __Type},
    registry,
    registry::is_visible,
    Context, Object,
};

pub struct __Field<'a> {
    pub registry: &'a registry::Registry,
    pub visible_types: &'a HashSet<&'a str>,
    pub field: &'a registry::MetaField,
}

/// Object and Interface types are described by a list of Fields, each of which
/// has a name, potentially a list of arguments, and a return type.
#[Object(internal, name = "__Field")]
impl<'a> __Field<'a> {
    #[inline]
    async fn name(&self) -> &str {
        &self.field.name
    }

    #[inline]
    async fn description(&self) -> Option<&str> {
        self.field.description.as_deref()
    }

    async fn args(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = false)] include_deprecated: bool,
    ) -> Vec<__InputValue<'a>> {
        self.field
            .args
            .values()
            .filter(|input_value| include_deprecated || !input_value.deprecation.is_deprecated())
            .filter(|input_value| is_visible(ctx, &input_value.visible))
            .map(|input_value| __InputValue {
                registry: self.registry,
                visible_types: self.visible_types,
                input_value,
            })
            .collect()
    }

    #[graphql(name = "type")]
    async fn ty(&self) -> __Type<'a> {
        __Type::new(self.registry, self.visible_types, &self.field.ty)
    }

    #[inline]
    async fn is_deprecated(&self) -> bool {
        self.field.deprecation.is_deprecated()
    }

    #[inline]
    async fn deprecation_reason(&self) -> Option<&str> {
        self.field.deprecation.reason()
    }
}
