use crate::{registry, Context, ContextSelectionSet, Result};
use graphql_parser::query::{Field, Value};
use std::borrow::Cow;

/// Represents a GraphQL type
pub trait GQLType {
    /// Type the name.
    fn type_name() -> Cow<'static, str>;

    /// Qualified typename.
    fn qualified_type_name() -> String {
        format!("{}!", Self::type_name())
    }

    /// Create type information in the registry and return qualified typename.
    fn create_type_info(registry: &mut registry::Registry) -> String;
}

/// Represents a GraphQL input value
pub trait GQLInputValue: GQLType + Sized {
    fn parse(value: &Value) -> Option<Self>;
}

/// Represents a GraphQL output value
#[async_trait::async_trait]
pub trait GQLOutputValue: GQLType {
    async fn resolve(value: &Self, ctx: &ContextSelectionSet<'_>) -> Result<serde_json::Value>;
}

/// Represents a GraphQL object
#[async_trait::async_trait]
pub trait GQLObject: GQLOutputValue {
    /// This function returns true of type `GQLEmptyMutation` only
    #[doc(hidden)]
    fn is_empty() -> bool {
        return false;
    }

    /// Resolves a field value and outputs it as a json value `serde_json::Value`.
    async fn resolve_field(&self, ctx: &Context<'_>, field: &Field) -> Result<serde_json::Value>;

    /// Resolve an inline fragment with the `name`.
    async fn resolve_inline_fragment(
        &self,
        name: &str,
        ctx: &ContextSelectionSet<'_>,
        result: &mut serde_json::Map<String, serde_json::Value>,
    ) -> Result<()>;
}

/// Represents a GraphQL input object
pub trait GQLInputObject: GQLInputValue {}

/// Represents a GraphQL scalar
///
/// You can implement the trait to create a custom scalar.
///
/// # Examples
///
/// ```rust
/// use async_graphql::*;
///
/// struct MyInt(i32);
///
/// impl GQLScalar for MyInt {
///     fn type_name() -> &'static str {
///         "MyInt"
///     }
///
///     fn parse(value: &Value) -> Option<Self> {
///         if let Value::Int(n) = value {
///             Some(MyInt(n.as_i64().unwrap() as i32))
///         } else {
///             None
///         }
///     }
///
///     fn to_json(&self) -> Result<serde_json::Value> {
///         Ok(self.0.into())
///     }
/// }
///
/// impl_scalar!(MyInt); // // Don't forget this one
/// ```
pub trait GQLScalar: Sized + Send {
    /// The type name of a scalar.
    fn type_name() -> &'static str;

    /// The description of a scalar.
    fn description() -> Option<&'static str> {
        None
    }

    /// Parse a scalar value, return `Some(Self)` if successful, otherwise return `None`.
    fn parse(value: &Value) -> Option<Self>;

    /// Checks for a valid scalar value.
    ///
    /// The default implementation is to try to parse it, and in some cases you can implement this on your own to improve performance.
    fn is_valid(value: &Value) -> bool {
        Self::parse(value).is_some()
    }

    /// Convert the scalar value to json value.
    fn to_json(&self) -> Result<serde_json::Value>;
}

#[macro_export]
#[doc(hidden)]
macro_rules! impl_scalar_internal {
    ($ty:ty) => {
        impl crate::GQLType for $ty {
            fn type_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(<$ty as crate::GQLScalar>::type_name())
            }

            fn create_type_info(registry: &mut crate::registry::Registry) -> String {
                registry.create_type::<$ty, _>(|_| crate::registry::Type::Scalar {
                    name: <$ty as crate::GQLScalar>::type_name().to_string(),
                    description: <$ty>::description(),
                    is_valid: |value| <$ty as crate::GQLScalar>::is_valid(value),
                })
            }
        }

        impl crate::GQLType for &$ty {
            fn type_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(<$ty as crate::GQLScalar>::type_name())
            }

            fn create_type_info(registry: &mut crate::registry::Registry) -> String {
                registry.create_type::<$ty, _>(|_| crate::registry::Type::Scalar {
                    name: <$ty as crate::GQLScalar>::type_name().to_string(),
                    description: <$ty>::description(),
                    is_valid: |value| <$ty as crate::GQLScalar>::is_valid(value),
                })
            }
        }

        impl crate::GQLInputValue for $ty {
            fn parse(value: &crate::Value) -> Option<Self> {
                <$ty as crate::GQLScalar>::parse(value)
            }
        }

        #[async_trait::async_trait]
        impl crate::GQLOutputValue for $ty {
            async fn resolve(
                value: &Self,
                _: &crate::ContextSelectionSet<'_>,
            ) -> crate::Result<serde_json::Value> {
                value.to_json()
            }
        }

        #[async_trait::async_trait]
        impl crate::GQLOutputValue for &$ty {
            async fn resolve(
                value: &Self,
                _: &crate::ContextSelectionSet<'_>,
            ) -> crate::Result<serde_json::Value> {
                value.to_json()
            }
        }
    };
}

#[macro_export]
macro_rules! impl_scalar {
    ($ty:ty) => {
        impl async_graphql::GQLType for $ty {
            fn type_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(<$ty as async_graphql::GQLScalar>::type_name())
            }

            fn create_type_info(registry: &mut async_graphql::registry::Registry) -> String {
                registry.create_type::<$ty, _>(|_| async_graphql::registry::Type::Scalar {
                    name: <$ty as async_graphql::GQLScalar>::type_name().to_string(),
                    description: <$ty>::description(),
                    is_valid: |value| <$ty as async_graphql::GQLScalar>::is_valid(value),
                })
            }
        }

        impl async_graphql::GQLType for &$ty {
            fn type_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(<$ty as async_graphql::GQLScalar>::type_name())
            }

            fn create_type_info(registry: &mut async_graphql::registry::Registry) -> String {
                registry.create_type::<$ty, _>(|_| async_graphql::registry::Type::Scalar {
                    name: <$ty as async_graphql::GQLScalar>::type_name().to_string(),
                    description: <$ty>::description(),
                    is_valid: |value| <$ty as async_graphql::GQLScalar>::is_valid(value),
                })
            }
        }

        impl async_graphql::GQLInputValue for $ty {
            fn parse(value: &async_graphql::Value) -> Option<Self> {
                <$ty as async_graphql::GQLScalar>::parse(value)
            }
        }

        #[async_graphql::async_trait::async_trait]
        impl async_graphql::GQLOutputValue for $ty {
            async fn resolve(
                value: &Self,
                _: &async_graphql::ContextSelectionSet<'_>,
            ) -> async_graphql::Result<serde_json::Value> {
                value.to_json()
            }
        }

        #[async_graphql::async_trait::async_trait]
        impl async_graphql::GQLOutputValue for &$ty {
            async fn resolve(
                value: &Self,
                _: &async_graphql::ContextSelectionSet<'_>,
            ) -> async_graphql::Result<serde_json::Value> {
                value.to_json()
            }
        }
    };
}

/// Represents a GraphQL output value
#[async_trait::async_trait]
impl<T: GQLObject + Send + Sync> GQLOutputValue for T {
    async fn resolve(value: &Self, ctx: &ContextSelectionSet<'_>) -> Result<serde_json::Value> {
        let mut result = serde_json::Map::<String, serde_json::Value>::new();
        crate::resolver::do_resolve(ctx, value, &mut result).await?;
        Ok(result.into())
    }
}
