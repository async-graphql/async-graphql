mod cache_control;
mod export_sdl;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use indexmap::map::IndexMap;
use indexmap::set::IndexSet;

use crate::parser::types::{BaseType as ParsedBaseType, Type as ParsedType};
use crate::validators::InputValueValidator;
use crate::{model, Any, Context, Type, Value};

pub use cache_control::CacheControl;

fn strip_brackets(type_name: &str) -> Option<&str> {
    if let Some(rest) = type_name.strip_prefix('[') {
        Some(&rest[..rest.len() - 1])
    } else {
        None
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MetaTypeName<'a> {
    List(&'a str),
    NonNull(&'a str),
    Named(&'a str),
}

impl<'a> std::fmt::Display for MetaTypeName<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetaTypeName::Named(name) => write!(f, "{}", name),
            MetaTypeName::NonNull(name) => write!(f, "{}!", name),
            MetaTypeName::List(name) => write!(f, "[{}]", name),
        }
    }
}

impl<'a> MetaTypeName<'a> {
    pub fn create(type_name: &str) -> MetaTypeName {
        if let Some(type_name) = type_name.strip_suffix('!') {
            MetaTypeName::NonNull(type_name)
        } else if let Some(type_name) = strip_brackets(type_name) {
            MetaTypeName::List(type_name)
        } else {
            MetaTypeName::Named(type_name)
        }
    }

    pub fn concrete_typename(type_name: &str) -> &str {
        match MetaTypeName::create(type_name) {
            MetaTypeName::List(type_name) => Self::concrete_typename(type_name),
            MetaTypeName::NonNull(type_name) => Self::concrete_typename(type_name),
            MetaTypeName::Named(type_name) => type_name,
        }
    }

    pub fn is_non_null(&self) -> bool {
        matches!(self, MetaTypeName::NonNull(_))
    }

    pub fn unwrap_non_null(&self) -> Self {
        match self {
            MetaTypeName::NonNull(ty) => MetaTypeName::create(ty),
            _ => *self,
        }
    }

    pub fn is_subtype(&self, sub: &MetaTypeName<'_>) -> bool {
        match (self, sub) {
            (MetaTypeName::NonNull(super_type), MetaTypeName::NonNull(sub_type))
            | (MetaTypeName::Named(super_type), MetaTypeName::NonNull(sub_type)) => {
                MetaTypeName::create(super_type).is_subtype(&MetaTypeName::create(sub_type))
            }
            (MetaTypeName::Named(super_type), MetaTypeName::Named(sub_type)) => {
                super_type == sub_type
            }
            (MetaTypeName::List(super_type), MetaTypeName::List(sub_type)) => {
                MetaTypeName::create(super_type).is_subtype(&MetaTypeName::create(sub_type))
            }
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct MetaInputValue {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub ty: String,
    pub default_value: Option<String>,
    pub validator: Option<Arc<dyn InputValueValidator>>,
    pub visible: Option<MetaVisibleFn>,
}

#[derive(Clone)]
pub struct MetaField {
    pub name: String,
    pub description: Option<&'static str>,
    pub args: IndexMap<&'static str, MetaInputValue>,
    pub ty: String,
    pub deprecation: Option<&'static str>,
    pub cache_control: CacheControl,
    pub external: bool,
    pub requires: Option<&'static str>,
    pub provides: Option<&'static str>,
    pub visible: Option<MetaVisibleFn>,
}

#[derive(Clone)]
pub struct MetaEnumValue {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub deprecation: Option<&'static str>,
    pub visible: Option<MetaVisibleFn>,
}

type MetaVisibleFn = fn(&Context<'_>) -> bool;

pub enum MetaType {
    Scalar {
        name: String,
        description: Option<&'static str>,
        is_valid: fn(value: &Value) -> bool,
        visible: Option<MetaVisibleFn>,
    },
    Object {
        name: String,
        description: Option<&'static str>,
        fields: IndexMap<String, MetaField>,
        cache_control: CacheControl,
        extends: bool,
        keys: Option<Vec<String>>,
        visible: Option<MetaVisibleFn>,
    },
    Interface {
        name: String,
        description: Option<&'static str>,
        fields: IndexMap<String, MetaField>,
        possible_types: IndexSet<String>,
        extends: bool,
        keys: Option<Vec<String>>,
        visible: Option<MetaVisibleFn>,
    },
    Union {
        name: String,
        description: Option<&'static str>,
        possible_types: IndexSet<String>,
        visible: Option<MetaVisibleFn>,
    },
    Enum {
        name: String,
        description: Option<&'static str>,
        enum_values: IndexMap<&'static str, MetaEnumValue>,
        visible: Option<MetaVisibleFn>,
    },
    InputObject {
        name: String,
        description: Option<&'static str>,
        input_fields: IndexMap<String, MetaInputValue>,
        visible: Option<MetaVisibleFn>,
    },
}

impl MetaType {
    pub fn field_by_name(&self, name: &str) -> Option<&MetaField> {
        self.fields().and_then(|fields| fields.get(name))
    }

    pub fn fields(&self) -> Option<&IndexMap<String, MetaField>> {
        match self {
            MetaType::Object { fields, .. } => Some(&fields),
            MetaType::Interface { fields, .. } => Some(&fields),
            _ => None,
        }
    }

    pub fn is_visible(&self, ctx: &Context<'_>) -> bool {
        let visible = match self {
            MetaType::Scalar { visible, .. } => visible,
            MetaType::Object { visible, .. } => visible,
            MetaType::Interface { visible, .. } => visible,
            MetaType::Union { visible, .. } => visible,
            MetaType::Enum { visible, .. } => visible,
            MetaType::InputObject { visible, .. } => visible,
        };
        match visible {
            Some(f) => f(ctx),
            None => true,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            MetaType::Scalar { name, .. } => &name,
            MetaType::Object { name, .. } => name,
            MetaType::Interface { name, .. } => name,
            MetaType::Union { name, .. } => name,
            MetaType::Enum { name, .. } => name,
            MetaType::InputObject { name, .. } => name,
        }
    }

    pub fn is_composite(&self) -> bool {
        match self {
            MetaType::Object { .. } => true,
            MetaType::Interface { .. } => true,
            MetaType::Union { .. } => true,
            _ => false,
        }
    }

    pub fn is_abstract(&self) -> bool {
        match self {
            MetaType::Interface { .. } => true,
            MetaType::Union { .. } => true,
            _ => false,
        }
    }

    pub fn is_leaf(&self) -> bool {
        match self {
            MetaType::Enum { .. } => true,
            MetaType::Scalar { .. } => true,
            _ => false,
        }
    }

    pub fn is_input(&self) -> bool {
        match self {
            MetaType::Enum { .. } => true,
            MetaType::Scalar { .. } => true,
            MetaType::InputObject { .. } => true,
            _ => false,
        }
    }

    pub fn is_possible_type(&self, type_name: &str) -> bool {
        match self {
            MetaType::Interface { possible_types, .. } => possible_types.contains(type_name),
            MetaType::Union { possible_types, .. } => possible_types.contains(type_name),
            MetaType::Object { name, .. } => name == type_name,
            _ => false,
        }
    }

    pub fn possible_types(&self) -> Option<&IndexSet<String>> {
        match self {
            MetaType::Interface { possible_types, .. } => Some(possible_types),
            MetaType::Union { possible_types, .. } => Some(possible_types),
            _ => None,
        }
    }

    pub fn type_overlap(&self, ty: &MetaType) -> bool {
        if self as *const MetaType == ty as *const MetaType {
            return true;
        }

        match (self.is_abstract(), ty.is_abstract()) {
            (true, true) => self
                .possible_types()
                .iter()
                .copied()
                .flatten()
                .any(|type_name| ty.is_possible_type(type_name)),
            (true, false) => self.is_possible_type(ty.name()),
            (false, true) => ty.is_possible_type(self.name()),
            (false, false) => false,
        }
    }
}

pub struct MetaDirective {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub locations: Vec<model::__DirectiveLocation>,
    pub args: IndexMap<&'static str, MetaInputValue>,
}

#[derive(Default)]
pub struct Registry {
    pub types: IndexMap<String, MetaType>,
    pub directives: HashMap<String, MetaDirective>,
    pub implements: HashMap<String, HashSet<String>>,
    pub query_type: String,
    pub mutation_type: Option<String>,
    pub subscription_type: Option<String>,
}

impl Registry {
    pub fn create_type<T: crate::Type, F: FnMut(&mut Registry) -> MetaType>(
        &mut self,
        mut f: F,
    ) -> String {
        let name = T::type_name();
        if !self.types.contains_key(name.as_ref()) {
            // Inserting a fake type before calling the function allows recursive types to exist.
            self.types.insert(
                name.clone().into_owned(),
                MetaType::Object {
                    name: "".to_string(),
                    description: None,
                    fields: Default::default(),
                    cache_control: Default::default(),
                    extends: false,
                    keys: None,
                    visible: None,
                },
            );
            let ty = f(self);
            *self.types.get_mut(&*name).unwrap() = ty;
        }
        T::qualified_type_name()
    }

    pub fn create_dummy_type<T: crate::Type>(&mut self) -> MetaType {
        let mut dummy_registry = Registry::default();
        T::create_type_info(&mut dummy_registry);
        if let Some(ty) = dummy_registry.types.remove(&*T::type_name()) {
            // Do not overwrite existing types.
            for (name, ty) in dummy_registry.types {
                if !self.types.contains_key(&name) {
                    self.types.insert(name, ty);
                }
            }

            // Do not overwrite existing implements.
            for (name, interfaces) in dummy_registry.implements {
                if let Some(current_interfaces) = self.implements.get_mut(&name) {
                    current_interfaces.extend(interfaces);
                } else {
                    self.implements.insert(name, interfaces);
                }
            }

            ty
        } else {
            unreachable!()
        }
    }

    pub fn add_directive(&mut self, directive: MetaDirective) {
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

    pub fn add_keys(&mut self, ty: &str, keys: &str) {
        let all_keys = match self.types.get_mut(ty) {
            Some(MetaType::Object { keys: all_keys, .. }) => all_keys,
            Some(MetaType::Interface { keys: all_keys, .. }) => all_keys,
            _ => return,
        };
        if let Some(all_keys) = all_keys {
            all_keys.push(keys.to_string());
        } else {
            *all_keys = Some(vec![keys.to_string()]);
        }
    }

    pub fn concrete_type_by_name(&self, type_name: &str) -> Option<&MetaType> {
        self.types.get(MetaTypeName::concrete_typename(type_name))
    }

    pub fn concrete_type_by_parsed_type(&self, query_type: &ParsedType) -> Option<&MetaType> {
        match &query_type.base {
            ParsedBaseType::Named(name) => self.types.get(name.as_str()),
            ParsedBaseType::List(ty) => self.concrete_type_by_parsed_type(ty),
        }
    }

    pub(crate) fn has_entities(&self) -> bool {
        self.types.values().any(|ty| match ty {
            MetaType::Object {
                keys: Some(keys), ..
            } => !keys.is_empty(),
            MetaType::Interface {
                keys: Some(keys), ..
            } => !keys.is_empty(),
            _ => false,
        })
    }

    fn create_entity_type(&mut self) {
        let possible_types = self
            .types
            .values()
            .filter_map(|ty| match ty {
                MetaType::Object {
                    name,
                    keys: Some(keys),
                    ..
                } if !keys.is_empty() => Some(name.clone()),
                MetaType::Interface {
                    name,
                    keys: Some(keys),
                    ..
                } if !keys.is_empty() => Some(name.clone()),
                _ => None,
            })
            .collect();

        self.types.insert(
            "_Entity".to_string(),
            MetaType::Union {
                name: "_Entity".to_string(),
                description: None,
                possible_types,
                visible: None,
            },
        );
    }

    pub(crate) fn create_federation_types(&mut self) {
        Any::create_type_info(self);

        self.types.insert(
            "_Service".to_string(),
            MetaType::Object {
                name: "_Service".to_string(),
                description: None,
                fields: {
                    let mut fields = IndexMap::new();
                    fields.insert(
                        "sdl".to_string(),
                        MetaField {
                            name: "sdl".to_string(),
                            description: None,
                            args: Default::default(),
                            ty: "String".to_string(),
                            deprecation: None,
                            cache_control: Default::default(),
                            external: false,
                            requires: None,
                            provides: None,
                            visible: None,
                        },
                    );
                    fields
                },
                cache_control: Default::default(),
                extends: false,
                keys: None,
                visible: None,
            },
        );

        self.create_entity_type();

        let query_root = self.types.get_mut(&self.query_type).unwrap();
        if let MetaType::Object { fields, .. } = query_root {
            fields.insert(
                "_service".to_string(),
                MetaField {
                    name: "_service".to_string(),
                    description: None,
                    args: Default::default(),
                    ty: "_Service!".to_string(),
                    deprecation: None,
                    cache_control: Default::default(),
                    external: false,
                    requires: None,
                    provides: None,
                    visible: None,
                },
            );

            fields.insert(
                "_entities".to_string(),
                MetaField {
                    name: "_entities".to_string(),
                    description: None,
                    args: {
                        let mut args = IndexMap::new();
                        args.insert(
                            "representations",
                            MetaInputValue {
                                name: "representations",
                                description: None,
                                ty: "[_Any!]!".to_string(),
                                default_value: None,
                                validator: None,
                                visible: None,
                            },
                        );
                        args
                    },
                    ty: "[_Entity]!".to_string(),
                    deprecation: None,
                    cache_control: Default::default(),
                    external: false,
                    requires: None,
                    provides: None,
                    visible: None,
                },
            );
        }
    }

    pub fn names(&self) -> Vec<String> {
        let mut names = HashSet::new();

        for d in self.directives.values() {
            names.insert(d.name.to_string());
            names.extend(d.args.values().map(|arg| arg.name.to_string()));
        }

        for ty in self.types.values() {
            match ty {
                MetaType::Scalar { name, .. } | MetaType::Union { name, .. } => {
                    names.insert(name.clone());
                }
                MetaType::Object { name, fields, .. }
                | MetaType::Interface { name, fields, .. } => {
                    names.insert(name.clone());
                    names.extend(
                        fields
                            .values()
                            .map(|field| {
                                std::iter::once(field.name.clone())
                                    .chain(field.args.values().map(|arg| arg.name.to_string()))
                            })
                            .flatten(),
                    );
                }
                MetaType::Enum {
                    name, enum_values, ..
                } => {
                    names.insert(name.clone());
                    names.extend(enum_values.values().map(|value| value.name.to_string()));
                }
                MetaType::InputObject {
                    name, input_fields, ..
                } => {
                    names.insert(name.clone());
                    names.extend(input_fields.values().map(|field| field.name.to_string()));
                }
            }
        }

        names.into_iter().collect()
    }

    pub fn set_description<T: Type>(&mut self, desc: &'static str) {
        match self.types.get_mut(&*T::type_name()) {
            Some(MetaType::Scalar { description, .. }) => *description = Some(desc),
            Some(MetaType::Object { description, .. }) => *description = Some(desc),
            Some(MetaType::Interface { description, .. }) => *description = Some(desc),
            Some(MetaType::Union { description, .. }) => *description = Some(desc),
            Some(MetaType::Enum { description, .. }) => *description = Some(desc),
            Some(MetaType::InputObject { description, .. }) => *description = Some(desc),
            None => {}
        }
    }
}
