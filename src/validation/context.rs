use crate::error::RuleError;
use crate::registry::{Registry, Type};
use graphql_parser::query::{Definition, Document};
use graphql_parser::Pos;
use std::collections::HashSet;

pub struct ValidatorContext<'a> {
    pub registry: &'a Registry,
    pub errors: Vec<RuleError>,
    type_stack: Vec<&'a Type>,
    fragments_names: HashSet<&'a str>,
}

impl<'a> ValidatorContext<'a> {
    pub fn new(registry: &'a Registry, doc: &'a Document) -> Self {
        Self {
            registry,
            errors: Default::default(),
            type_stack: Default::default(),
            fragments_names: doc
                .definitions
                .iter()
                .filter_map(|d| match d {
                    Definition::Fragment(fragment) => Some(fragment.name.as_str()),
                    _ => None,
                })
                .collect(),
        }
    }

    pub fn report_error<T: Into<String>>(&mut self, locations: Vec<Pos>, msg: T) {
        self.errors.push(RuleError {
            locations,
            message: msg.into(),
        })
    }

    pub fn append_errors(&mut self, errors: Vec<RuleError>) {
        self.errors.extend(errors);
    }

    pub fn with_type<F: FnMut(&mut ValidatorContext<'a>)>(&mut self, ty: &'a Type, mut f: F) {
        self.type_stack.push(ty);
        f(self);
        self.type_stack.pop();
    }

    pub fn parent_type(&self) -> Option<&'a Type> {
        self.type_stack.get(self.type_stack.len() - 2).map(|t| *t)
    }

    pub fn current_type(&self) -> &'a Type {
        self.type_stack.last().unwrap()
    }

    pub fn is_known_fragment(&self, name: &str) -> bool {
        self.fragments_names.contains(name)
    }
}
