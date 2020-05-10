use crate::model::{__InputValue, __Type};
use crate::registry;
use async_graphql_derive::Object;
use itertools::Itertools;

pub struct __Field<'a> {
    pub registry: &'a registry::Registry,
    pub field: &'a registry::Field,
}

/// Object and Interface types are described by a list of Fields, each of which has a name, potentially a list of arguments, and a return type.
#[Object(internal)]
impl<'a> __Field<'a> {
    async fn name(&self) -> String {
        self.field.name.to_string()
    }

    async fn description(&self) -> Option<String> {
        self.field.description.map(|s| s.to_string())
    }

    async fn args(&self) -> Vec<__InputValue<'a>> {
        let mut args = self
            .field
            .args
            .values()
            .map(|input_value| __InputValue {
                registry: self.registry,
                input_value,
            })
            .collect_vec();
        args.sort_by(|a, b| a.input_value.name.cmp(b.input_value.name));
        args
    }

    #[field(name = "type")]
    async fn ty(&self) -> __Type<'a> {
        __Type::new(self.registry, &self.field.ty)
    }

    async fn is_deprecated(&self) -> bool {
        self.field.deprecation.is_some()
    }

    async fn deprecation_reason(&self) -> Option<String> {
        self.field.deprecation.map(|s| s.to_string())
    }
}
