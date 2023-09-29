use std::{
    borrow::Cow,
    fmt::{self, Display},
};

/// A type reference
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TypeRef {
    /// Named type
    Named(Cow<'static, str>),
    /// Non-null type
    NonNull(Box<TypeRef>),
    /// List type
    List(Box<TypeRef>),
}

impl Display for TypeRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeRef::Named(name) => write!(f, "{}", name),
            TypeRef::NonNull(ty) => write!(f, "{}!", ty),
            TypeRef::List(ty) => write!(f, "[{}]", ty),
        }
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

    /// A Upload type
    pub const UPLOAD: &'static str = "Upload";

    /// Returns the nullable type reference
    ///
    /// GraphQL Type: `T`
    #[inline]
    pub fn named(type_name: impl Into<String>) -> TypeRef {
        TypeRef::Named(type_name.into().into())
    }

    /// Returns the non-null type reference
    ///
    /// GraphQL Type: `T!`
    #[inline]
    pub fn named_nn(type_name: impl Into<String>) -> TypeRef {
        TypeRef::NonNull(Box::new(TypeRef::Named(type_name.into().into())))
    }

    /// Returns a nullable list of nullable members type reference
    ///
    /// GraphQL Type: `[T]`
    #[inline]
    pub fn named_list(type_name: impl Into<String>) -> TypeRef {
        TypeRef::List(Box::new(TypeRef::Named(type_name.into().into())))
    }

    /// Returns a nullable list of non-null members type reference
    ///
    /// GraphQL Type: `[T!]`
    #[inline]
    pub fn named_nn_list(type_name: impl Into<String>) -> TypeRef {
        TypeRef::List(Box::new(TypeRef::NonNull(Box::new(TypeRef::Named(
            type_name.into().into(),
        )))))
    }

    /// Returns a non-null list of nullable members type reference
    ///
    /// GraphQL Type: `[T]!`
    #[inline]
    pub fn named_list_nn(type_name: impl Into<String>) -> TypeRef {
        TypeRef::NonNull(Box::new(TypeRef::List(Box::new(TypeRef::Named(
            type_name.into().into(),
        )))))
    }

    /// Returns a non-null list of non-null members type reference
    ///
    /// GraphQL Type: `[T!]!`
    #[inline]
    pub fn named_nn_list_nn(type_name: impl Into<String>) -> TypeRef {
        TypeRef::NonNull(Box::new(TypeRef::List(Box::new(TypeRef::NonNull(
            Box::new(TypeRef::Named(type_name.into().into())),
        )))))
    }

    /// Returns the type name
    ///
    /// `[Foo!]` -> `Foo`
    #[inline(always)]
    pub fn type_name(&self) -> &str {
        match self {
            TypeRef::Named(name) => name,
            TypeRef::NonNull(inner) => inner.type_name(),
            TypeRef::List(inner) => inner.type_name(),
        }
    }

    #[inline]
    pub(crate) fn is_nullable(&self) -> bool {
        match self {
            TypeRef::Named(_) => true,
            TypeRef::NonNull(_) => false,
            TypeRef::List(_) => true,
        }
    }

    pub(crate) fn is_subtype(&self, sub: &TypeRef) -> bool {
        fn is_subtype(cur: &TypeRef, sub: &TypeRef) -> bool {
            match (cur, sub) {
                (TypeRef::NonNull(super_type), TypeRef::NonNull(sub_type)) => {
                    is_subtype(&super_type, &sub_type)
                }
                (_, TypeRef::NonNull(sub_type)) => is_subtype(cur, &sub_type),
                (TypeRef::Named(super_type), TypeRef::Named(sub_type)) => super_type == sub_type,
                (TypeRef::List(super_type), TypeRef::List(sub_type)) => {
                    is_subtype(super_type, sub_type)
                }
                _ => false,
            }
        }

        is_subtype(self, sub)
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
