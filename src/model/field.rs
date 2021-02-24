use crate::model::{__InputValue, __Type};
use crate::{registry, Context, Object};

pub struct __Field<'a> {
    pub registry: &'a registry::Registry,
    pub field: &'a registry::MetaField,
}

/// Object and Interface types are described by a list of Fields, each of which has a name, potentially a list of arguments, and a return type.
#[Object(internal, name = "__Field")]
impl<'a> __Field<'a> {
    async fn name(&self) -> &str {
        &self.field.name
    }

    async fn description(&self) -> Option<&str> {
        self.field.description
    }

    async fn args(&self, ctx: &Context<'_>) -> Vec<__InputValue<'a>> {
        self.field
            .args
            .values()
            .filter(|input_value| match &input_value.visible {
                Some(f) => f(ctx),
                None => true,
            })
            .map(|input_value| __InputValue {
                registry: self.registry,
                input_value,
            })
            .collect()
    }

    #[graphql(name = "type")]
    async fn ty(&self) -> __Type<'a> {
        __Type::new(self.registry, &self.field.ty)
    }

    async fn is_deprecated(&self) -> bool {
        self.field.deprecation.is_some()
    }

    async fn deprecation_reason(&self) -> Option<&str> {
        self.field.deprecation
    }
}
