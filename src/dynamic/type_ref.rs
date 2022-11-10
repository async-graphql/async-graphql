use std::{
    borrow::Cow,
    fmt::{self, Display},
};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) enum TypeRefInner {
    Named(Cow<'static, str>),
    NonNull(Box<TypeRefInner>),
    List(Box<TypeRefInner>),
}

impl Display for TypeRefInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeRefInner::Named(name) => write!(f, "{}", name),
            TypeRefInner::NonNull(ty) => write!(f, "{}!", ty),
            TypeRefInner::List(ty) => write!(f, "[{}]", ty),
        }
    }
}

impl TypeRefInner {
    fn type_name(&self) -> &str {
        match self {
            TypeRefInner::Named(name) => name,
            TypeRefInner::NonNull(inner) => inner.type_name(),
            TypeRefInner::List(inner) => inner.type_name(),
        }
    }
}

/// A type reference
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TypeRef(pub(crate) TypeRefInner);

impl Display for TypeRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A type reference builder for named type
pub struct NamedTypeRefBuilder {
    name: Cow<'static, str>,
    non_null: bool,
}

impl NamedTypeRefBuilder {
    /// Specifies this type is non-null
    pub fn non_null(self) -> Self {
        Self {
            non_null: true,
            ..self
        }
    }

    /// Consumes this type and returns a list type builder
    pub fn list(self) -> ListTypeRefBuilder {
        ListTypeRefBuilder {
            element: TypeRef::from(self).0,
            non_null: false,
        }
    }

    /// Consumes this builder and returns a type reference
    pub fn into_type_ref(self) -> TypeRef {
        TypeRef::from(self)
    }
}

impl From<NamedTypeRefBuilder> for TypeRef {
    fn from(builder: NamedTypeRefBuilder) -> Self {
        let ty = TypeRefInner::Named(builder.name);
        if builder.non_null {
            TypeRef(TypeRefInner::NonNull(Box::new(ty)))
        } else {
            TypeRef(ty)
        }
    }
}

/// A type reference builder for list type
pub struct ListTypeRefBuilder {
    element: TypeRefInner,
    non_null: bool,
}

impl ListTypeRefBuilder {
    /// Specifies this list is non-null
    pub fn non_null(self) -> Self {
        Self {
            non_null: true,
            ..self
        }
    }

    /// Consumes this builder and returns a type reference
    pub fn into_type_ref(self) -> TypeRef {
        TypeRef::from(self)
    }
}

impl From<ListTypeRefBuilder> for TypeRef {
    fn from(builder: ListTypeRefBuilder) -> Self {
        if builder.non_null {
            TypeRef(TypeRefInner::NonNull(Box::new(TypeRefInner::List(
                Box::new(builder.element),
            ))))
        } else {
            TypeRef(TypeRefInner::List(Box::new(builder.element)))
        }
    }
}

impl TypeRef {
    /// A Int scalar type
    pub const INT: NamedTypeRefBuilder = NamedTypeRefBuilder {
        name: Cow::Borrowed("Int"),
        non_null: false,
    };
    /// A Float scalar type
    pub const FLOAT: NamedTypeRefBuilder = NamedTypeRefBuilder {
        name: Cow::Borrowed("Float"),
        non_null: false,
    };
    /// A String scalar type
    pub const STRING: NamedTypeRefBuilder = NamedTypeRefBuilder {
        name: Cow::Borrowed("String"),
        non_null: false,
    };
    /// A Boolean scalar type
    pub const BOOLEAN: NamedTypeRefBuilder = NamedTypeRefBuilder {
        name: Cow::Borrowed("Boolean"),
        non_null: false,
    };
    /// A ID scalar type
    pub const ID: NamedTypeRefBuilder = NamedTypeRefBuilder {
        name: Cow::Borrowed("ID"),
        non_null: false,
    };

    /// Create a named type reference
    #[inline(always)]
    pub fn named(type_name: impl Into<Cow<'static, str>>) -> NamedTypeRefBuilder {
        NamedTypeRefBuilder {
            name: type_name.into(),
            non_null: false,
        }
    }

    #[inline(always)]
    pub(crate) fn type_name(&self) -> &str {
        self.0.type_name()
    }

    #[inline]
    pub(crate) fn is_nullable(&self) -> bool {
        match &self.0 {
            TypeRefInner::Named(_) => true,
            TypeRefInner::NonNull(_) => false,
            TypeRefInner::List(_) => true,
        }
    }

    #[inline]
    pub(crate) fn is_named(&self) -> bool {
        match &self.0 {
            TypeRefInner::Named(_) => true,
            TypeRefInner::NonNull(_) => false,
            TypeRefInner::List(_) => false,
        }
    }

    #[inline]
    pub(crate) fn is_list(&self) -> bool {
        match &self.0 {
            TypeRefInner::Named(_) => false,
            TypeRefInner::NonNull(_) => false,
            TypeRefInner::List(_) => true,
        }
    }

    pub(crate) fn is_subtype(&self, sub: &TypeRef) -> bool {
        fn is_subtype(cur: &TypeRefInner, sub: &TypeRefInner) -> bool {
            match (cur, sub) {
                (TypeRefInner::NonNull(super_type), TypeRefInner::NonNull(sub_type)) => {
                    is_subtype(&super_type, &sub_type)
                }
                (_, TypeRefInner::NonNull(sub_type)) => is_subtype(cur, &sub_type),
                (TypeRefInner::Named(super_type), TypeRefInner::Named(sub_type)) => {
                    super_type == sub_type
                }
                (TypeRefInner::List(super_type), TypeRefInner::List(sub_type)) => {
                    is_subtype(super_type, sub_type)
                }
                _ => false,
            }
        }

        is_subtype(&self.0, &sub.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        assert_eq!(TypeRef::named("MyObj").into_type_ref().to_string(), "MyObj");
        assert_eq!(
            TypeRef::named("MyObj").list().into_type_ref().to_string(),
            "[MyObj]"
        );
        assert_eq!(
            TypeRef::named("MyObj")
                .non_null()
                .list()
                .into_type_ref()
                .to_string(),
            "[MyObj!]"
        );
        assert_eq!(
            TypeRef::named("MyObj")
                .non_null()
                .list()
                .non_null()
                .into_type_ref()
                .to_string(),
            "[MyObj!]!"
        );
    }
}
