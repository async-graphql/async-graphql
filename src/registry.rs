use crate::{model, GQLType, Value};
use std::collections::{HashMap, HashSet};

fn parse_non_null(type_name: &str) -> Option<&str> {
    if type_name.ends_with("!") {
        Some(&type_name[..type_name.len() - 1])
    } else {
        None
    }
}

fn parse_list(type_name: &str) -> Option<&str> {
    if type_name.starts_with("[") {
        Some(&type_name[1..type_name.len() - 1])
    } else {
        None
    }
}

pub enum TypeName<'a> {
    List(&'a str),
    NonNull(&'a str),
    Named(&'a str),
}

impl<'a> TypeName<'a> {
    pub fn create(type_name: &str) -> TypeName {
        if let Some(type_name) = parse_non_null(type_name) {
            TypeName::NonNull(type_name)
        } else if let Some(type_name) = parse_list(type_name) {
            TypeName::List(type_name)
        } else {
            TypeName::Named(type_name)
        }
    }

    pub fn get_basic_typename(type_name: &str) -> &str {
        match TypeName::create(type_name) {
            TypeName::List(type_name) => Self::get_basic_typename(type_name),
            TypeName::NonNull(type_name) => Self::get_basic_typename(type_name),
            TypeName::Named(type_name) => type_name,
        }
    }
}

pub struct InputValue {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub ty: String,
    pub default_value: Option<&'static str>,
}

pub struct Field {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub args: HashMap<&'static str, InputValue>,
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
        is_valid: fn(value: &Value) -> bool,
    },
    Object {
        name: &'static str,
        description: Option<&'static str>,
        fields: HashMap<&'static str, Field>,
    },
    Interface {
        name: &'static str,
        description: Option<&'static str>,
        fields: HashMap<&'static str, Field>,
        possible_types: Vec<String>,
    },
    Union {
        name: &'static str,
        description: Option<&'static str>,
        possible_types: Vec<String>,
    },
    Enum {
        name: &'static str,
        description: Option<&'static str>,
        enum_values: HashMap<&'static str, EnumValue>,
    },
    InputObject {
        name: &'static str,
        description: Option<&'static str>,
        input_fields: Vec<InputValue>,
    },
}

impl Type {
    pub fn field_by_name(&self, name: &str) -> Option<&Field> {
        self.fields().and_then(|fields| fields.get(name))
    }

    pub fn fields(&self) -> Option<&HashMap<&'static str, Field>> {
        match self {
            Type::Object { fields, .. } => Some(&fields),
            Type::Interface { fields, .. } => Some(&fields),
            _ => None,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Type::Scalar { name, .. } => &name,
            Type::Object { name, .. } => name,
            Type::Interface { name, .. } => name,
            Type::Union { name, .. } => name,
            Type::Enum { name, .. } => name,
            Type::InputObject { name, .. } => name,
        }
    }

    pub fn is_composite(&self) -> bool {
        match self {
            Type::Object { .. } => true,
            Type::Interface { .. } => true,
            Type::Union { .. } => true,
            _ => false,
        }
    }

    pub fn is_leaf(&self) -> bool {
        match self {
            Type::Enum { .. } => true,
            Type::Scalar { .. } => true,
            _ => false,
        }
    }

    pub fn is_input(&self) -> bool {
        match self {
            Type::Enum { .. } => true,
            Type::Scalar { .. } => true,
            Type::InputObject { .. } => true,
            _ => false,
        }
    }
}

pub struct Directive {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub locations: Vec<model::__DirectiveLocation>,
    pub args: HashMap<&'static str, InputValue>,
}

pub struct Registry {
    pub types: HashMap<String, Type>,
    pub directives: HashMap<String, Directive>,
    pub implements: HashMap<String, HashSet<String>>,
    pub query_type: String,
    pub mutation_type: Option<String>,
}

impl Registry {
    pub fn create_type<T: GQLType, F: FnMut(&mut Registry) -> Type>(&mut self, mut f: F) -> String {
        let name = T::type_name();
        if !self.types.contains_key(name.as_ref()) {
            self.types.insert(
                name.to_string(),
                Type::Object {
                    name: "",
                    description: None,
                    fields: Default::default(),
                },
            );
            let mut ty = f(self);
            if let Type::Object { fields, .. } = &mut ty {
                fields.insert(
                    "__typename",
                    Field {
                        name: "__typename",
                        description: None,
                        args: Default::default(),
                        ty: "String!".to_string(),
                        deprecation: None,
                    },
                );
            }
            self.types.insert(name.to_string(), ty);
        }
        T::qualified_type_name()
    }

    pub fn add_directive(&mut self, directive: Directive) {
        self.directives
            .insert(directive.name.to_string(), directive);
    }

    pub fn add_implements(&mut self, ty: &str, interface: &str) {
        self.implements
            .entry(ty.to_string())
            .and_modify(|interfaces| {
                interfaces.insert(interface.to_string());
            })
            .or_insert({
                let mut interfaces = HashSet::new();
                interfaces.insert(interface.to_string());
                interfaces
            });
    }

    pub fn get_basic_type(&self, type_name: &str) -> Option<&Type> {
        self.types.get(TypeName::get_basic_typename(type_name))
    }
}
