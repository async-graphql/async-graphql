use crate::model::{__InputValue, __Type};
use crate::registry;
use async_graphql_derive::Object;

pub struct __Field<'a> {
    pub registry: &'a registry::Registry,
    pub field: &'a registry::Field,
}

#[Object(
    internal,
    desc = "Object and Interface types are described by a list of Fields, each of which has a name, potentially a list of arguments, and a return type."
)]
impl<'a> __Field<'a> {
    #[field]
    fn name(&self) -> String {
        self.field.name.to_string()
    }

    #[field]
    fn description(&self) -> Option<String> {
        self.field.description.map(|s| s.to_string())
    }

    #[field]
    fn args(&self) -> Vec<__InputValue<'a>> {
        let mut args = self
            .field
            .args
            .values()
            .map(|input_value| __InputValue {
                registry: self.registry,
                input_value,
            })
            .collect::<Vec<_>>();
        args.sort_by(|a, b| a.input_value.name.cmp(b.input_value.name));
        args
    }

    #[field(name = "type")]
    fn ty(&self) -> __Type<'a> {
        __Type::new(self.registry, &self.field.ty)
    }

    #[field]
    fn is_deprecated(&self) -> bool {
        self.field.deprecation.is_some()
    }

    #[field]
    fn deprecation_reason(&self) -> Option<String> {
        self.field.deprecation.map(|s| s.to_string())
    }
}
