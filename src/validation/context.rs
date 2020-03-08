use crate::error::RuleError;
use crate::registry::{Registry, Type};
use graphql_parser::Pos;

pub struct ValidatorContext<'a> {
    pub registry: &'a Registry,
    pub errors: Vec<RuleError>,
    type_stack: Vec<&'a Type>,
}

impl<'a> ValidatorContext<'a> {
    pub fn new(registry: &'a Registry) -> Self {
        Self {
            registry,
            errors: Default::default(),
            type_stack: Default::default(),
        }
    }

    pub fn report_error<T: Into<String>>(&mut self, locations: Vec<Pos>, msg: T) {
        self.errors.push(RuleError {
            locations,
            message: msg.into(),
        })
    }

    pub fn with_type<F: FnMut(&mut ValidatorContext<'a>)>(&mut self, ty: &'a Type, mut f: F) {
        self.type_stack.push(ty);
        f(self);
        self.type_stack.pop();
    }

    pub fn parent_type(&self) -> &'a Type {
        self.type_stack.last().unwrap()
    }
}
