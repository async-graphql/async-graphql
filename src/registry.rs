use crate::{model, GQLType};
use std::collections::HashMap;

pub struct InputValue {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub ty: String,
    pub default_value: Option<&'static str>,
}

pub struct Field {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub args: Vec<InputValue>,
    pub ty: String,
    pub deprecation: Option<&'static str>,
}

pub struct EnumValue {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub deprecation: Option<&'static str>,
}

pub enum Type {
    Scalar {
        name: String,
        description: Option<&'static str>,
    },
    Object {
        name: &'static str,
        description: Option<&'static str>,
        fields: Vec<Field>,
    },
    Interface {
        name: &'static str,
        description: Option<&'static str>,
        fields: Vec<Field>,
        possible_types: Vec<usize>,
    },
    Union {
        name: &'static str,
        description: Option<&'static str>,
        possible_types: Vec<usize>,
    },
    Enum {
        name: &'static str,
        description: Option<&'static str>,
        enum_values: Vec<EnumValue>,
    },
    InputObject {
        name: &'static str,
        description: Option<&'static str>,
        input_fields: Vec<InputValue>,
    },
}

pub struct Directive {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub locations: Vec<model::__DirectiveLocation>,
    pub args: Vec<InputValue>,
}

#[derive(Default)]
pub struct Registry {
    pub types: HashMap<String, Type>,
    pub directives: Vec<Directive>,
}

impl Registry {
    pub fn create_type<T: GQLType, F: FnMut(&mut Registry) -> Type>(&mut self, mut f: F) -> String {
        let name = T::type_name();
        if !self.types.contains_key(name.as_ref()) {
            let ty = f(self);
            self.types.insert(name.to_string(), ty);
        }
        T::qualified_type_name()
    }

    pub fn add_directive(&mut self, directive: Directive) {
        self.directives.push(directive);
    }
}
