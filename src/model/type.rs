use crate::model::{__EnumValue, __Field, __InputValue, __TypeKind};
use crate::registry;
use async_graphql_derive::Object;
use itertools::Itertools;

enum TypeDetail<'a> {
    Named(&'a registry::MetaType),
    NonNull(String),
    List(String),
}

pub struct __Type<'a> {
    registry: &'a registry::Registry,
    detail: TypeDetail<'a>,
}

impl<'a> __Type<'a> {
    pub fn new_simple(registry: &'a registry::Registry, ty: &'a registry::MetaType) -> __Type<'a> {
        __Type {
            registry,
            detail: TypeDetail::Named(ty),
        }
    }

    pub fn new(registry: &'a registry::Registry, type_name: &str) -> __Type<'a> {
        match registry::MetaTypeName::create(type_name) {
            registry::MetaTypeName::NonNull(ty) => __Type {
                registry,
                detail: TypeDetail::NonNull(ty.to_string()),
            },
            registry::MetaTypeName::List(ty) => __Type {
                registry,
                detail: TypeDetail::List(ty.to_string()),
            },
            registry::MetaTypeName::Named(ty) => __Type {
                registry,
                detail: TypeDetail::Named(&registry.types[ty]),
            },
        }
    }
}

/// The fundamental unit of any GraphQL Schema is the type. There are many kinds of types in GraphQL as represented by the `__TypeKind` enum.
///
/// Depending on the kind of a type, certain fields describe information about that type. Scalar types provide no information beyond a name and description, while Enum types provide their values. Object and Interface types provide the fields they describe. Abstract types, Union and Interface, provide the Object types possible at runtime. List and NonNull types compose other types.
#[Object(internal)]
impl<'a> __Type<'a> {
    async fn kind(&self) -> __TypeKind {
        match &self.detail {
            TypeDetail::Named(ty) => match ty {
                registry::MetaType::Scalar { .. } => __TypeKind::Scalar,
                registry::MetaType::Object { .. } => __TypeKind::Object,
                registry::MetaType::Interface { .. } => __TypeKind::Interface,
                registry::MetaType::Union { .. } => __TypeKind::Union,
                registry::MetaType::Enum { .. } => __TypeKind::Enum,
                registry::MetaType::InputObject { .. } => __TypeKind::InputObject,
            },
            TypeDetail::NonNull(_) => __TypeKind::NonNull,
            TypeDetail::List(_) => __TypeKind::List,
        }
    }

    async fn name(&self) -> Option<String> {
        match &self.detail {
            TypeDetail::Named(ty) => Some(ty.name().to_string()),
            TypeDetail::NonNull(_) => None,
            TypeDetail::List(_) => None,
        }
    }

    async fn description(&self) -> Option<String> {
        match &self.detail {
            TypeDetail::Named(ty) => match ty {
                registry::MetaType::Scalar { description, .. } => {
                    description.map(|s| s.to_string())
                }
                registry::MetaType::Object { description, .. } => {
                    description.map(|s| s.to_string())
                }
                registry::MetaType::Interface { description, .. } => {
                    description.map(|s| s.to_string())
                }
                registry::MetaType::Union { description, .. } => description.map(|s| s.to_string()),
                registry::MetaType::Enum { description, .. } => description.map(|s| s.to_string()),
                registry::MetaType::InputObject { description, .. } => {
                    description.map(|s| s.to_string())
                }
            },
            TypeDetail::NonNull(_) => None,
            TypeDetail::List(_) => None,
        }
    }

    async fn fields(
        &self,
        #[arg(default = false)] include_deprecated: bool,
    ) -> Option<Vec<__Field<'a>>> {
        if let TypeDetail::Named(ty) = &self.detail {
            ty.fields().map(|fields| {
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
                    .collect_vec()
            })
        } else {
            None
        }
    }

    async fn interfaces(&self) -> Option<Vec<__Type<'a>>> {
        if let TypeDetail::Named(registry::MetaType::Object { name, .. }) = &self.detail {
            Some(
                self.registry
                    .implements
                    .get(name)
                    .unwrap_or(&Default::default())
                    .iter()
                    .map(|ty| __Type::new(self.registry, ty))
                    .collect(),
            )
        } else {
            None
        }
    }

    async fn possible_types(&self) -> Option<Vec<__Type<'a>>> {
        if let TypeDetail::Named(registry::MetaType::Interface { possible_types, .. }) =
            &self.detail
        {
            Some(
                possible_types
                    .iter()
                    .map(|ty| __Type::new(self.registry, ty))
                    .collect(),
            )
        } else if let TypeDetail::Named(registry::MetaType::Union { possible_types, .. }) =
            &self.detail
        {
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

    async fn enum_values(
        &self,
        #[arg(default = false)] include_deprecated: bool,
    ) -> Option<Vec<__EnumValue<'a>>> {
        if let TypeDetail::Named(registry::MetaType::Enum { enum_values, .. }) = &self.detail {
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

    async fn input_fields(&self) -> Option<Vec<__InputValue<'a>>> {
        if let TypeDetail::Named(registry::MetaType::InputObject { input_fields, .. }) =
            &self.detail
        {
            Some(
                input_fields
                    .values()
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
