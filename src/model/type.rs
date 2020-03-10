use crate::model::{__EnumValue, __Field, __InputValue, __TypeKind};
use crate::registry;
use crate::registry::{Type, TypeName};
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
        match TypeName::create(type_name) {
            TypeName::NonNull(ty) => __Type {
                registry,
                detail: TypeDetail::NonNull(ty.to_string()),
            },
            TypeName::List(ty) => __Type {
                registry,
                detail: TypeDetail::List(ty.to_string()),
            },
            TypeName::Named(ty) => __Type {
                registry,
                detail: TypeDetail::Simple(&registry.types[ty]),
            },
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
                registry::Type::Scalar { .. } => __TypeKind::Scalar,
                registry::Type::Object { .. } => __TypeKind::Object,
                registry::Type::Interface { .. } => __TypeKind::Interface,
                registry::Type::Union { .. } => __TypeKind::Union,
                registry::Type::Enum { .. } => __TypeKind::Enum,
                registry::Type::InputObject { .. } => __TypeKind::InputObject,
            },
            TypeDetail::NonNull(_) => __TypeKind::NonNull,
            TypeDetail::List(_) => __TypeKind::List,
        }
    }

    #[field]
    async fn name(&self) -> Option<String> {
        match &self.detail {
            TypeDetail::Simple(ty) => Some(ty.name().to_string()),
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
        #[arg(default = "false")] include_deprecated: bool,
    ) -> Option<Vec<__Field<'a>>> {
        if let TypeDetail::Simple(ty) = &self.detail {
            ty.fields().and_then(|fields| {
                Some(
                    fields
                        .values()
                        .filter(|field| {
                            (include_deprecated || field.deprecation.is_none())
                                && !field.name.starts_with("__")
                        })
                        .map(|field| __Field {
                            registry: self.registry,
                            field,
                        })
                        .collect(),
                )
            })
        } else {
            None
        }
    }

    #[field]
    async fn interfaces(&self) -> Option<Vec<__Type<'a>>> {
        if let TypeDetail::Simple(Type::Object { name, .. }) = &self.detail {
            Some(
                self.registry
                    .implements
                    .get(*name)
                    .unwrap_or(&Default::default())
                    .iter()
                    .map(|ty| __Type::new(self.registry, ty))
                    .collect(),
            )
        } else {
            None
        }
    }

    #[field]
    async fn possible_types(&self) -> Option<Vec<__Type<'a>>> {
        if let TypeDetail::Simple(Type::Interface { possible_types, .. }) = &self.detail {
            Some(
                possible_types
                    .iter()
                    .map(|ty| __Type::new(self.registry, ty))
                    .collect(),
            )
        } else if let TypeDetail::Simple(Type::Union { possible_types, .. }) = &self.detail {
            Some(
                possible_types
                    .iter()
                    .map(|ty| __Type::new(self.registry, ty))
                    .collect(),
            )
        } else {
            None
        }
    }

    #[field]
    async fn enum_values(
        &self,
        #[arg(default = "false")] include_deprecated: bool,
    ) -> Option<Vec<__EnumValue<'a>>> {
        if let TypeDetail::Simple(Type::Enum { enum_values, .. }) = &self.detail {
            Some(
                enum_values
                    .values()
                    .filter(|field| include_deprecated || field.deprecation.is_none())
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

    #[field]
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

    #[field]
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
