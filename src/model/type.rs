use crate::model::{__EnumValue, __Field, __InputValue, __TypeKind};
use crate::registry;
use crate::registry::Type;
use async_graphql_derive::Object;

enum TypeDetail<'a> {
    Simple(&'a registry::Type),
    NonNull(String),
    List(String),
}

pub struct __Type<'a> {
    registry: &'a registry::Registry,
    detail: TypeDetail<'a>,
}

impl<'a> __Type<'a> {
    pub fn new_simple(registry: &'a registry::Registry, ty: &'a registry::Type) -> __Type<'a> {
        __Type {
            registry,
            detail: TypeDetail::Simple(ty),
        }
    }

    pub fn new(registry: &'a registry::Registry, type_name: &str) -> __Type<'a> {
        if let Some(type_name) = parse_non_null(type_name) {
            __Type {
                registry,
                detail: TypeDetail::NonNull(type_name.to_string()),
            }
        } else if let Some(type_name) = parse_list(type_name) {
            __Type {
                registry,
                detail: TypeDetail::List(type_name.to_string()),
            }
        } else {
            __Type {
                registry,
                detail: TypeDetail::Simple(&registry.types[type_name]),
            }
        }
    }
}

#[Object(
    internal,
    desc = r#"
The fundamental unit of any GraphQL Schema is the type. There are many kinds of types in GraphQL as represented by the `__TypeKind` enum.

Depending on the kind of a type, certain fields describe information about that type. Scalar types provide no information beyond a name and description, while Enum types provide their values. Object and Interface types provide the fields they describe. Abstract types, Union and Interface, provide the Object types possible at runtime. List and NonNull types compose other types.
"#
)]
impl<'a> __Type<'a> {
    #[field]
    async fn kind(&self) -> __TypeKind {
        match &self.detail {
            TypeDetail::Simple(ty) => match ty {
                registry::Type::Scalar { .. } => __TypeKind::SCALAR,
                registry::Type::Object { .. } => __TypeKind::OBJECT,
                registry::Type::Interface { .. } => __TypeKind::INTERFACE,
                registry::Type::Union { .. } => __TypeKind::UNION,
                registry::Type::Enum { .. } => __TypeKind::ENUM,
                registry::Type::InputObject { .. } => __TypeKind::INPUT_OBJECT,
            },
            TypeDetail::NonNull(_) => __TypeKind::NON_NULL,
            TypeDetail::List(_) => __TypeKind::LIST,
        }
    }

    #[field]
    async fn name(&self) -> Option<String> {
        match &self.detail {
            TypeDetail::Simple(ty) => match ty {
                registry::Type::Scalar { name, .. } => Some(name.clone()),
                registry::Type::Object { name, .. } => Some(name.to_string()),
                registry::Type::Interface { name, .. } => Some(name.to_string()),
                registry::Type::Union { name, .. } => Some(name.to_string()),
                registry::Type::Enum { name, .. } => Some(name.to_string()),
                registry::Type::InputObject { name, .. } => Some(name.to_string()),
            },
            TypeDetail::NonNull(_) => None,
            TypeDetail::List(_) => None,
        }
    }

    #[field]
    async fn description(&self) -> Option<String> {
        match &self.detail {
            TypeDetail::Simple(ty) => match ty {
                registry::Type::Scalar { description, .. } => description.map(|s| s.to_string()),
                registry::Type::Object { description, .. } => description.map(|s| s.to_string()),
                registry::Type::Interface { description, .. } => description.map(|s| s.to_string()),
                registry::Type::Union { description, .. } => description.map(|s| s.to_string()),
                registry::Type::Enum { description, .. } => description.map(|s| s.to_string()),
                registry::Type::InputObject { description, .. } => {
                    description.map(|s| s.to_string())
                }
            },
            TypeDetail::NonNull(_) => None,
            TypeDetail::List(_) => None,
        }
    }

    #[field]
    async fn fields(
        &self,
        #[arg(name = "includeDeprecated", default = "false")] include_deprecated: bool,
    ) -> Option<Vec<__Field<'a>>> {
        if let TypeDetail::Simple(Type::Object { fields, .. }) = &self.detail {
            Some(
                fields
                    .iter()
                    .filter(|field| {
                        if include_deprecated {
                            true
                        } else {
                            field.deprecation.is_none()
                        }
                    })
                    .map(|field| __Field {
                        registry: self.registry,
                        field,
                    })
                    .collect(),
            )
        } else {
            None
        }
    }

    #[field]
    async fn interfaces(&self) -> Option<Vec<__Type<'a>>> {
        if let TypeDetail::Simple(Type::Object { .. }) = &self.detail {
            Some(vec![])
        } else {
            None
        }
    }

    #[field(name = "possibleTypes")]
    async fn possible_types(&self) -> Option<Vec<__Type<'a>>> {
        None
    }

    #[field(name = "enumValues")]
    async fn enum_values(
        &self,
        #[arg(name = "includeDeprecated", default = "false")] include_deprecated: bool,
    ) -> Option<Vec<__EnumValue<'a>>> {
        if let TypeDetail::Simple(Type::Enum { enum_values, .. }) = &self.detail {
            Some(
                enum_values
                    .iter()
                    .filter(|field| {
                        if include_deprecated {
                            true
                        } else {
                            field.deprecation.is_none()
                        }
                    })
                    .map(|value| __EnumValue {
                        registry: self.registry,
                        value,
                    })
                    .collect(),
            )
        } else {
            None
        }
    }

    #[field(name = "inputFields")]
    async fn input_fields(&self) -> Option<Vec<__InputValue<'a>>> {
        if let TypeDetail::Simple(Type::InputObject { input_fields, .. }) = &self.detail {
            Some(
                input_fields
                    .iter()
                    .map(|input_value| __InputValue {
                        registry: self.registry,
                        input_value,
                    })
                    .collect(),
            )
        } else {
            None
        }
    }

    #[field(name = "ofType")]
    async fn of_type(&self) -> Option<__Type<'a>> {
        if let TypeDetail::List(ty) = &self.detail {
            Some(__Type::new(self.registry, &ty))
        } else if let TypeDetail::NonNull(ty) = &self.detail {
            Some(__Type::new(self.registry, &ty))
        } else {
            None
        }
    }
}

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
