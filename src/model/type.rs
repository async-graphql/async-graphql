use crate::model::{__EnumValue, __Field, __InputValue, __TypeKind};
use crate::{registry, Context, Object};

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
    #[inline]
    pub fn new_simple(registry: &'a registry::Registry, ty: &'a registry::MetaType) -> __Type<'a> {
        __Type {
            registry,
            detail: TypeDetail::Named(ty),
        }
    }

    #[inline]
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
#[Object(internal, name = "__Type")]
impl<'a> __Type<'a> {
    #[inline]
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

    #[inline]
    async fn name(&self) -> Option<&str> {
        match &self.detail {
            TypeDetail::Named(ty) => Some(ty.name()),
            TypeDetail::NonNull(_) => None,
            TypeDetail::List(_) => None,
        }
    }

    #[inline]
    async fn description(&self) -> Option<&str> {
        match &self.detail {
            TypeDetail::Named(ty) => match ty {
                registry::MetaType::Scalar { description, .. }
                | registry::MetaType::Object { description, .. }
                | registry::MetaType::Interface { description, .. }
                | registry::MetaType::Union { description, .. }
                | registry::MetaType::Enum { description, .. }
                | registry::MetaType::InputObject { description, .. } => description.as_deref(),
            },
            TypeDetail::NonNull(_) => None,
            TypeDetail::List(_) => None,
        }
    }

    async fn fields(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = false)] include_deprecated: bool,
    ) -> Option<Vec<__Field<'a>>> {
        if let TypeDetail::Named(ty) = &self.detail {
            ty.fields().map(|fields| {
                fields
                    .values()
                    .filter(|field| match &field.visible {
                        Some(f) => f(ctx),
                        None => true,
                    })
                    .filter(|field| {
                        (include_deprecated || !field.deprecation.is_deprecated())
                            && !field.name.starts_with("__")
                    })
                    .map(|field| __Field {
                        registry: self.registry,
                        field,
                    })
                    .collect()
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
        ctx: &Context<'_>,
        #[graphql(default = false)] include_deprecated: bool,
    ) -> Option<Vec<__EnumValue<'a>>> {
        if let TypeDetail::Named(registry::MetaType::Enum { enum_values, .. }) = &self.detail {
            Some(
                enum_values
                    .values()
                    .filter(|value| match &value.visible {
                        Some(f) => f(ctx),
                        None => true,
                    })
                    .filter(|value| include_deprecated || !value.deprecation.is_deprecated())
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

    async fn input_fields(&self, ctx: &Context<'_>) -> Option<Vec<__InputValue<'a>>> {
        if let TypeDetail::Named(registry::MetaType::InputObject { input_fields, .. }) =
            &self.detail
        {
            Some(
                input_fields
                    .values()
                    .filter(|input_value| match &input_value.visible {
                        Some(f) => f(ctx),
                        None => true,
                    })
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

    #[inline]
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
