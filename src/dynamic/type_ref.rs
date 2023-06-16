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

impl TypeRef {
    /// A int scalar type
    pub const INT: &'static str = "Int";

    /// A float scalar type
    pub const FLOAT: &'static str = "Float";

    /// A string scalar type
    pub const STRING: &'static str = "String";

    /// A boolean scalar type
    pub const BOOLEAN: &'static str = "Boolean";

    /// A ID scalar type
    pub const ID: &'static str = "ID";

    /// Returns the nullable type reference
    ///
    /// GraphQL Type: `T`
    #[inline]
    pub fn named(type_name: impl Into<String>) -> TypeRef {
        TypeRef(TypeRefInner::Named(type_name.into().into()))
    }

    /// Returns the non-null type reference
    ///
    /// GraphQL Type: `T!`
    #[inline]
    pub fn named_nn(type_name: impl Into<String>) -> TypeRef {
        TypeRef(TypeRefInner::NonNull(Box::new(TypeRefInner::Named(
            type_name.into().into(),
        ))))
    }

    /// Returns a nullable list of nullable members type reference
    ///
    /// GraphQL Type: `[T]`
    #[inline]
    pub fn named_list(type_name: impl Into<String>) -> TypeRef {
        TypeRef(TypeRefInner::List(Box::new(TypeRefInner::Named(
            type_name.into().into(),
        ))))
    }

    /// Returns a nullable list of non-null members type reference
    ///
    /// GraphQL Type: `[T!]`
    #[inline]
    pub fn named_nn_list(type_name: impl Into<String>) -> TypeRef {
        TypeRef(TypeRefInner::List(Box::new(TypeRefInner::NonNull(
            Box::new(TypeRefInner::Named(type_name.into().into())),
        ))))
    }

    /// Returns a non-null list of nullable members type reference
    ///
    /// GraphQL Type: `[T]!`
    #[inline]
    pub fn named_list_nn(type_name: impl Into<String>) -> TypeRef {
        TypeRef(TypeRefInner::NonNull(Box::new(TypeRefInner::List(
            Box::new(TypeRefInner::Named(type_name.into().into())),
        ))))
    }

    /// Returns a non-null list of non-null members type reference
    ///
    /// GraphQL Type: `[T!]!`
    #[inline]
    pub fn named_nn_list_nn(type_name: impl Into<String>) -> TypeRef {
        TypeRef(TypeRefInner::NonNull(Box::new(TypeRefInner::List(
            Box::new(TypeRefInner::NonNull(Box::new(TypeRefInner::Named(
                type_name.into().into(),
            )))),
        ))))
    }

    #[inline(always)]
    pub fn type_name(&self) -> &str {
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
        assert_eq!(TypeRef::named("MyObj").to_string(), "MyObj");
        assert_eq!(TypeRef::named_nn("MyObj").to_string(), "MyObj!");
        assert_eq!(TypeRef::named_list("MyObj").to_string(), "[MyObj]");
        assert_eq!(TypeRef::named_list_nn("MyObj").to_string(), "[MyObj]!");
        assert_eq!(TypeRef::named_nn_list("MyObj").to_string(), "[MyObj!]");
        assert_eq!(TypeRef::named_nn_list_nn("MyObj").to_string(), "[MyObj!]!");
    }
}
