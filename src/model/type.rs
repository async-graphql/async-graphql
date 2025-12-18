use std::collections::HashSet;

use crate::{
    Context, Object,
    model::{__EnumValue, __Field, __InputValue, __TypeKind},
    registry,
    registry::is_visible,
};

enum TypeDetail<'a> {
    Named(&'a registry::MetaType),
    NonNull(String),
    List(String),
}


pub struct __Type<'a> {
    registry: &'a registry::Registry,
    visible_types: &'a HashSet<&'a str>,
    detail: TypeDetail<'a>,
}

impl<'a> __Type<'a> {
    #[inline]
    pub fn new_simple(
        registry: &'a registry::Registry,
        visible_types: &'a HashSet<&'a str>,
        ty: &'a registry::MetaType,
    ) -> __Type<'a> {
        __Type {
            registry,
            visible_types,
            detail: TypeDetail::Named(ty),
        }
    }

    #[inline]
    pub fn new(
        registry: &'a registry::Registry,
        visible_types: &'a HashSet<&'a str>,
        type_name: &str,
    ) -> __Type<'a> {
        match registry::MetaTypeName::create(type_name) {
            registry::MetaTypeName::NonNull(ty) => __Type {
                registry,
                visible_types,
                detail: TypeDetail::NonNull(ty.to_string()),
            },
            registry::MetaTypeName::List(ty) => __Type {
                registry,
                visible_types,
                detail: TypeDetail::List(ty.to_string()),
            },
            registry::MetaTypeName::Named(ty) => __Type {
                registry,
                visible_types,
                detail: TypeDetail::Named(match registry.types.get(ty) {
                    Some(t) => t,
                    None => panic!("Type '{}' not found!", ty),
                }),
            },
        }
    }
}

/// The fundamental unit of any GraphQL Schema is the type. There are many kinds
/// of types in GraphQL as represented by the `__TypeKind` enum.
///
/// Depending on the kind of a type, certain fields describe information about
/// that type. Scalar types provide no information beyond a name and
/// description, while Enum types provide their values. Object and Interface
/// types provide the fields they describe. Abstract types, Union and Interface,
/// provide the Object types possible at runtime. List and NonNull types compose
/// other types.
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
                    .filter(|field| is_visible(ctx, &field.visible))
                    .filter(|field| {
                        (include_deprecated || !field.deprecation.is_deprecated())
                            && !field.name.starts_with("__")
                    })
                    .map(|field| __Field {
                        registry: self.registry,
                        visible_types: self.visible_types,
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
                    .filter(|ty| self.visible_types.contains(ty.as_str()))
                    .map(|ty| __Type::new(self.registry, self.visible_types, ty))
                    .collect(),
            )
        } else {
            None
        }
    }

    async fn possible_types(&self) -> Option<Vec<__Type<'a>>> {
        if let TypeDetail::Named(registry::MetaType::Interface { possible_types, .. })
        | TypeDetail::Named(registry::MetaType::Union { possible_types, .. }) = &self.detail
        {
            Some(
                possible_types
                    .iter()
                    .filter(|ty| self.visible_types.contains(ty.as_str()))
                    .map(|ty| __Type::new(self.registry, self.visible_types, ty))
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
                    .filter(|value| is_visible(ctx, &value.visible))
                    .filter(|value| include_deprecated || !value.deprecation.is_deprecated())
                    .map(|value| __EnumValue { value })
                    .collect(),
            )
        } else {
            None
        }
    }

    async fn input_fields(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = false)] include_deprecated: bool,
    ) -> Option<Vec<__InputValue<'a>>> {
        if let TypeDetail::Named(registry::MetaType::InputObject { input_fields, .. }) =
            &self.detail
        {
            Some(
                input_fields
                    .values()
                    .filter(|input_value| {
                        include_deprecated || !input_value.deprecation.is_deprecated()
                    })
                    .filter(|input_value| is_visible(ctx, &input_value.visible))
                    .map(|input_value| __InputValue {
                        registry: self.registry,
                        visible_types: self.visible_types,
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
            Some(__Type::new(self.registry, self.visible_types, &ty))
        } else if let TypeDetail::NonNull(ty) = &self.detail {
            Some(__Type::new(self.registry, self.visible_types, &ty))
        } else {
            None
        }
    }

    #[graphql(name = "specifiedByURL")]
    async fn specified_by_url(&self) -> Option<&'a str> {
        if let TypeDetail::Named(registry::MetaType::Scalar {
            specified_by_url, ..
        }) = &self.detail
        {
            specified_by_url.as_deref()
        } else {
            None
        }
    }

    async fn is_one_of(&self) -> Option<bool> {
        if let TypeDetail::Named(registry::MetaType::InputObject { oneof, .. }) = &self.detail {
            Some(*oneof)
        } else {
            None
        }
    }
}
