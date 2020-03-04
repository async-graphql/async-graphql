use crate::model::{__EnumValue, __Field, __InputValue, __TypeKind};
use crate::registry::Type;
use crate::{registry, Context, Result};
use async_graphql_derive::Object;

enum TypeDetail<'a> {
    Simple(&'a registry::Type),
    NonNull(&'a registry::Type),
    List(&'a registry::Type),
}

#[Object(
    internal,
    desc = r#"
The fundamental unit of any GraphQL Schema is the type. There are many kinds of types in GraphQL as represented by the `__TypeKind` enum.

Depending on the kind of a type, certain fields describe information about that type. Scalar types provide no information beyond a name and description, while Enum types provide their values. Object and Interface types provide the fields they describe. Abstract types, Union and Interface, provide the Object types possible at runtime. List and NonNull types compose other types.
"#,
    field(name = "kind", type = "__TypeKind", owned),
    field(name = "name", type = "Option<String>", owned),
    field(name = "description", type = "Option<String>", owned),
    field(
        name = "fields",
        type = "Option<Vec<__Field>>",
        owned,
        arg(name = "includeDeprecated", type = "bool", default = "false")
    ),
    field(name = "interfaces", type = "Option<Vec<__Type>>", owned),
    field(name = "possibleTypes", type = "Option<Vec<__Type>>", owned),
    field(
        name = "enumValues",
        type = "Option<Vec<__EnumValue>>",
        owned,
        arg(name = "includeDeprecated", type = "bool", default = "false")
    ),
    field(name = "inputFields", type = "Option<Vec<__InputValue>>", owned),
    field(name = "ofType", type = "Option<__Type>", owned)
)]
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
                detail: TypeDetail::NonNull(&registry.types[type_name]),
            }
        } else if let Some(type_name) = parse_list(type_name) {
            __Type {
                registry,
                detail: TypeDetail::List(&registry.types[type_name]),
            }
        } else {
            __Type {
                registry,
                detail: TypeDetail::Simple(&registry.types[type_name]),
            }
        }
    }
}

#[async_trait::async_trait]
impl<'a> __TypeFields for __Type<'a> {
    async fn kind(&self, _: &Context<'_>) -> Result<__TypeKind> {
        match &self.detail {
            TypeDetail::Simple(ty) => Ok(match ty {
                registry::Type::Scalar { .. } => __TypeKind::SCALAR,
                registry::Type::Object { .. } => __TypeKind::OBJECT,
                registry::Type::Interface { .. } => __TypeKind::INTERFACE,
                registry::Type::Union { .. } => __TypeKind::UNION,
                registry::Type::Enum { .. } => __TypeKind::ENUM,
                registry::Type::InputObject { .. } => __TypeKind::INPUT_OBJECT,
            }),
            TypeDetail::NonNull(_) => Ok(__TypeKind::NON_NULL),
            TypeDetail::List(_) => Ok(__TypeKind::LIST),
        }
    }

    async fn name(&self, _: &Context<'_>) -> Result<Option<String>> {
        match &self.detail {
            TypeDetail::Simple(ty) => Ok(match ty {
                registry::Type::Scalar { name, .. } => Some(name.clone()),
                registry::Type::Object { name, .. } => Some(name.to_string()),
                registry::Type::Interface { name, .. } => Some(name.to_string()),
                registry::Type::Union { name, .. } => Some(name.to_string()),
                registry::Type::Enum { name, .. } => Some(name.to_string()),
                registry::Type::InputObject { name, .. } => Some(name.to_string()),
            }),
            TypeDetail::NonNull(_) => Ok(None),
            TypeDetail::List(_) => Ok(None),
        }
    }

    async fn description(&self, _: &Context<'_>) -> Result<Option<String>> {
        match &self.detail {
            TypeDetail::Simple(ty) => Ok(match ty {
                registry::Type::Scalar { description, .. } => description.map(|s| s.to_string()),
                registry::Type::Object { description, .. } => description.map(|s| s.to_string()),
                registry::Type::Interface { description, .. } => description.map(|s| s.to_string()),
                registry::Type::Union { description, .. } => description.map(|s| s.to_string()),
                registry::Type::Enum { description, .. } => description.map(|s| s.to_string()),
                registry::Type::InputObject { description, .. } => {
                    description.map(|s| s.to_string())
                }
            }),
            TypeDetail::NonNull(_) => Ok(None),
            TypeDetail::List(_) => Ok(None),
        }
    }

    async fn fields<'b>(
        &'b self,
        _: &Context<'_>,
        include_deprecated: bool,
    ) -> Result<Option<Vec<__Field<'b>>>> {
        if let TypeDetail::Simple(Type::Object { fields, .. }) = &self.detail {
            Ok(Some(
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
            ))
        } else {
            Ok(None)
        }
    }

    async fn interfaces<'b>(&'b self, _: &Context<'_>) -> Result<Option<Vec<__Type<'b>>>> {
        Ok(None)
    }

    async fn possible_types<'b>(&'b self, _: &Context<'_>) -> Result<Option<Vec<__Type<'b>>>> {
        Ok(None)
    }

    async fn enum_values<'b>(
        &'b self,
        _: &Context<'_>,
        include_deprecated: bool,
    ) -> Result<Option<Vec<__EnumValue<'b>>>> {
        if let TypeDetail::Simple(Type::Enum { enum_values, .. }) = &self.detail {
            Ok(Some(
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
            ))
        } else {
            Ok(None)
        }
    }

    async fn input_fields<'b>(&'b self, _: &Context<'_>) -> Result<Option<Vec<__InputValue<'b>>>> {
        if let TypeDetail::Simple(Type::InputObject { input_fields, .. }) = &self.detail {
            Ok(Some(
                input_fields
                    .iter()
                    .map(|input_value| __InputValue {
                        registry: self.registry,
                        input_value,
                    })
                    .collect(),
            ))
        } else {
            Ok(None)
        }
    }

    async fn of_type<'b>(&'b self, _: &Context<'_>) -> Result<Option<__Type<'b>>> {
        if let TypeDetail::List(ty) = &self.detail {
            Ok(Some(__Type {
                registry: self.registry,
                detail: TypeDetail::Simple(ty),
            }))
        } else if let TypeDetail::NonNull(ty) = &self.detail {
            Ok(Some(__Type {
                registry: self.registry,
                detail: TypeDetail::Simple(ty),
            }))
        } else {
            Ok(None)
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
