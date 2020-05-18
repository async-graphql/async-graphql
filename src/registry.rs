use crate::parser::query::Type as ParsedType;
use crate::validators::InputValueValidator;
use crate::{model, Any, Type as _, Value};
use indexmap::map::IndexMap;
use indexmap::set::IndexSet;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::sync::Arc;

fn parse_non_null(type_name: &str) -> Option<&str> {
    if type_name.ends_with('!') {
        Some(&type_name[..type_name.len() - 1])
    } else {
        None
    }
}

fn parse_list(type_name: &str) -> Option<&str> {
    if type_name.starts_with('[') {
        Some(&type_name[1..type_name.len() - 1])
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
        if let Some(type_name) = parse_non_null(type_name) {
            MetaTypeName::NonNull(type_name)
        } else if let Some(type_name) = parse_list(type_name) {
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
        if let MetaTypeName::NonNull(_) = self {
            true
        } else {
            false
        }
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
    pub default_value: Option<&'static str>,
    pub validator: Option<Arc<dyn InputValueValidator>>,
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
}

#[derive(Clone)]
pub struct MetaEnumValue {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub deprecation: Option<&'static str>,
}

/// Cache control values
///
/// # Examples
///
/// ```rust
/// use async_graphql::*;
///
/// struct QueryRoot;
///
/// #[Object(cache_control(max_age = 60))]
/// impl QueryRoot {
///     #[field(cache_control(max_age = 30))]
///     async fn value1(&self) -> i32 {
///         0
///     }
///
///     #[field(cache_control(private))]
///     async fn value2(&self) -> i32 {
///         0
///     }
/// }
///
/// #[async_std::main]
/// async fn main() {
///     let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
///     assert_eq!(schema.execute("{ value1 }").await.unwrap().cache_control, CacheControl { public: true, max_age: 30 });
///     assert_eq!(schema.execute("{ value2 }").await.unwrap().cache_control, CacheControl { public: false, max_age: 60 });
///     assert_eq!(schema.execute("{ value1 value2 }").await.unwrap().cache_control, CacheControl { public: false, max_age: 30 });
/// }
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CacheControl {
    /// Scope is public, default is true
    pub public: bool,

    /// Cache max age, default is 0.
    pub max_age: usize,
}

impl Default for CacheControl {
    fn default() -> Self {
        Self {
            public: true,
            max_age: 0,
        }
    }
}

impl CacheControl {
    /// Get 'Cache-Control' header value.
    pub fn value(&self) -> Option<String> {
        if self.max_age > 0 {
            if !self.public {
                Some(format!("max-age={}, private", self.max_age))
            } else {
                Some(format!("max-age={}", self.max_age))
            }
        } else {
            None
        }
    }
}

impl CacheControl {
    pub(crate) fn merge(&mut self, other: &CacheControl) {
        self.public = self.public && other.public;
        self.max_age = if self.max_age == 0 {
            other.max_age
        } else if other.max_age == 0 {
            self.max_age
        } else {
            self.max_age.min(other.max_age)
        };
    }
}

pub enum MetaType {
    Scalar {
        name: String,
        description: Option<&'static str>,
        is_valid: fn(value: &Value) -> bool,
    },
    Object {
        name: String,
        description: Option<&'static str>,
        fields: IndexMap<String, MetaField>,
        cache_control: CacheControl,
        extends: bool,
        keys: Option<Vec<String>>,
    },
    Interface {
        name: String,
        description: Option<&'static str>,
        fields: IndexMap<String, MetaField>,
        possible_types: IndexSet<String>,
        extends: bool,
        keys: Option<Vec<String>>,
    },
    Union {
        name: String,
        description: Option<&'static str>,
        possible_types: IndexSet<String>,
    },
    Enum {
        name: String,
        description: Option<&'static str>,
        enum_values: IndexMap<&'static str, MetaEnumValue>,
    },
    InputObject {
        name: String,
        description: Option<&'static str>,
        input_fields: IndexMap<String, MetaInputValue>,
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

pub struct Registry {
    pub types: HashMap<String, MetaType>,
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
            self.types.insert(
                name.to_string(),
                MetaType::Object {
                    name: "".to_string(),
                    description: None,
                    fields: Default::default(),
                    cache_control: Default::default(),
                    extends: false,
                    keys: None,
                },
            );
            let ty = f(self);
            self.types.insert(name.to_string(), ty);
        }
        T::qualified_type_name()
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
        match query_type {
            ParsedType::NonNull(ty) => self.concrete_type_by_parsed_type(ty),
            ParsedType::List(ty) => self.concrete_type_by_parsed_type(ty),
            ParsedType::Named(name) => self.types.get(name.as_str()),
        }
    }

    fn create_federation_fields<'a, I: Iterator<Item = &'a MetaField>>(sdl: &mut String, it: I) {
        for field in it {
            if field.name.starts_with("__") {
                continue;
            }
            if field.name == "_service" || field.name == "_entities" {
                continue;
            }

            if !field.args.is_empty() {
                write!(
                    sdl,
                    "\t{}({}): {}",
                    field.name,
                    field
                        .args
                        .values()
                        .map(|arg| federation_input_value(arg))
                        .join(""),
                    field.ty
                )
                .ok();
            } else {
                write!(sdl, "\t{}: {}", field.name, field.ty).ok();
            }

            if field.external {
                write!(sdl, " @external").ok();
            }
            if let Some(requires) = field.requires {
                write!(sdl, " @requires(fields: \"{}\")", requires).ok();
            }
            if let Some(provides) = field.provides {
                write!(sdl, " @provides(fields: \"{}\")", provides).ok();
            }
            writeln!(sdl).ok();
        }
    }

    fn create_federation_type(&self, ty: &MetaType, sdl: &mut String) {
        match ty {
            MetaType::Scalar { name, .. } => {
                const SYSTEM_SCALARS: &[&str] = &["Int", "Float", "String", "Boolean", "ID", "Any"];
                if !SYSTEM_SCALARS.contains(&name.as_str()) {
                    writeln!(sdl, "scalar {}", name).ok();
                }
            }
            MetaType::Object {
                name,
                fields,
                extends,
                keys,
                ..
            } => {
                if name.starts_with("__") {
                    return;
                }
                if name == "_Service" {
                    return;
                }
                if fields.len() == 4 {
                    // Is empty query root, only __schema, __type, _service, _entities fields
                    return;
                }

                if *extends {
                    write!(sdl, "extend ").ok();
                }
                write!(sdl, "type {} ", name).ok();
                if let Some(keys) = keys {
                    for key in keys {
                        write!(sdl, "@key(fields: \"{}\") ", key).ok();
                    }
                }
                writeln!(sdl, "{{").ok();
                Self::create_federation_fields(sdl, fields.values());
                writeln!(sdl, "}}").ok();
            }
            MetaType::Interface {
                name,
                fields,
                extends,
                keys,
                ..
            } => {
                if *extends {
                    write!(sdl, "extend ").ok();
                }
                write!(sdl, "interface {} ", name).ok();
                if let Some(keys) = keys {
                    for key in keys {
                        write!(sdl, "@key(fields: \"{}\") ", key).ok();
                    }
                }
                writeln!(sdl, "{{").ok();
                Self::create_federation_fields(sdl, fields.values());
                writeln!(sdl, "}}").ok();
            }
            MetaType::Enum {
                name, enum_values, ..
            } => {
                write!(sdl, "enum {} ", name).ok();
                writeln!(sdl, "{{").ok();
                for value in enum_values.values() {
                    writeln!(sdl, "{}", value.name).ok();
                }
                writeln!(sdl, "}}").ok();
            }
            MetaType::InputObject {
                name, input_fields, ..
            } => {
                write!(sdl, "input {} ", name).ok();
                writeln!(sdl, "{{").ok();
                for field in input_fields.values() {
                    writeln!(sdl, "{}", federation_input_value(&field)).ok();
                }
                writeln!(sdl, "}}").ok();
            }
            MetaType::Union {
                name,
                possible_types,
                ..
            } => {
                writeln!(
                    sdl,
                    "union {} = {}",
                    name,
                    possible_types.iter().join(" | ")
                )
                .ok();
            }
        }
    }

    pub fn create_federation_sdl(&self) -> String {
        let mut sdl = String::new();
        for ty in self.types.values() {
            if ty.name().starts_with("__") {
                continue;
            }
            const FEDERATION_TYPES: &[&str] = &["_Any", "_Entity", "_Service"];
            if FEDERATION_TYPES.contains(&ty.name()) {
                continue;
            }
            self.create_federation_type(ty, &mut sdl);
        }
        sdl
    }

    fn has_entities(&self) -> bool {
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
            },
        );
    }

    pub fn create_federation_types(&mut self) {
        if !self.has_entities() {
            return;
        }

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
                        },
                    );
                    fields
                },
                cache_control: Default::default(),
                extends: false,
                keys: None,
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
                },
            );
        }
    }
}

fn federation_input_value(input_value: &MetaInputValue) -> String {
    if let Some(default_value) = &input_value.default_value {
        format!(
            "{}: {} = {}",
            input_value.name, input_value.ty, default_value
        )
    } else {
        format!("{}: {}", input_value.name, input_value.ty)
    }
}
