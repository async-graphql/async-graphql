use crate::validators::InputValueValidator;
use crate::{model, Value};
use graphql_parser::query::Type as ParsedType;
use std::collections::{HashMap, HashSet};
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

    pub fn is_non_null(&self) -> bool {
        if let TypeName::NonNull(_) = self {
            true
        } else {
            false
        }
    }
}

#[derive(Clone)]
pub struct InputValue {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub ty: String,
    pub default_value: Option<&'static str>,
    pub validator: Option<Arc<dyn InputValueValidator>>,
}

#[derive(Clone)]
pub struct Field {
    pub name: String,
    pub description: Option<&'static str>,
    pub args: HashMap<&'static str, InputValue>,
    pub ty: String,
    pub deprecation: Option<&'static str>,
    pub cache_control: CacheControl,
}

#[derive(Clone)]
pub struct EnumValue {
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
///         unimplemented!()
///     }
///
///     #[field(cache_control(private))]
///     async fn value2(&self) -> i32 {
///         unimplemented!()
///     }
/// }
///
/// #[async_std::main]
/// async fn main() {
///     let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
///     assert_eq!(schema.query("{ value1 }").prepare().unwrap().cache_control(), CacheControl { public: true, max_age: 30 });
///     assert_eq!(schema.query("{ value2 }").prepare().unwrap().cache_control(), CacheControl { public: false, max_age: 60 });
///     assert_eq!(schema.query("{ value1 value2 }").prepare().unwrap().cache_control(), CacheControl { public: false, max_age: 30 });
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
    pub(crate) fn merge(self, other: &CacheControl) -> Self {
        CacheControl {
            public: self.public && other.public,
            max_age: if self.max_age == 0 {
                other.max_age
            } else if other.max_age == 0 {
                self.max_age
            } else {
                self.max_age.min(other.max_age)
            },
        }
    }
}

pub enum Type {
    Scalar {
        name: String,
        description: Option<&'static str>,
        is_valid: fn(value: &Value) -> bool,
    },
    Object {
        name: String,
        description: Option<&'static str>,
        fields: HashMap<String, Field>,
        cache_control: CacheControl,
    },
    Interface {
        name: String,
        description: Option<&'static str>,
        fields: HashMap<String, Field>,
        possible_types: HashSet<String>,
    },
    Union {
        name: String,
        description: Option<&'static str>,
        possible_types: HashSet<String>,
    },
    Enum {
        name: String,
        description: Option<&'static str>,
        enum_values: HashMap<&'static str, EnumValue>,
    },
    InputObject {
        name: String,
        description: Option<&'static str>,
        input_fields: Vec<InputValue>,
    },
}

impl Type {
    pub fn field_by_name(&self, name: &str) -> Option<&Field> {
        self.fields().and_then(|fields| fields.get(name))
    }

    pub fn fields(&self) -> Option<&HashMap<String, Field>> {
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

    pub fn is_possible_type(&self, type_name: &str) -> bool {
        match self {
            Type::Interface { possible_types, .. } => possible_types.contains(type_name),
            Type::Union { possible_types, .. } => possible_types.contains(type_name),
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
    pub subscription_type: Option<String>,
}

impl Registry {
    pub fn create_type<T: crate::Type, F: FnMut(&mut Registry) -> Type>(
        &mut self,
        mut f: F,
    ) -> String {
        let name = T::type_name();
        if !self.types.contains_key(name.as_ref()) {
            self.types.insert(
                name.to_string(),
                Type::Object {
                    name: "".to_string(),
                    description: None,
                    fields: Default::default(),
                    cache_control: Default::default(),
                },
            );
            let mut ty = f(self);
            if let Type::Object { fields, .. } = &mut ty {
                fields.insert(
                    "__typename".to_string(),
                    Field {
                        name: "__typename".to_string(),
                        description: None,
                        args: Default::default(),
                        ty: "String!".to_string(),
                        deprecation: None,
                        cache_control: Default::default(),
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

    pub fn basic_type_by_typename(&self, type_name: &str) -> Option<&Type> {
        self.types.get(TypeName::get_basic_typename(type_name))
    }

    pub fn basic_type_by_parsed_type(&self, query_type: &ParsedType) -> Option<&Type> {
        match query_type {
            ParsedType::NonNullType(ty) => self.basic_type_by_parsed_type(ty),
            ParsedType::ListType(ty) => self.basic_type_by_parsed_type(ty),
            ParsedType::NamedType(name) => self.types.get(name.as_str()),
        }
    }
}
