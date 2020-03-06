use crate::{registry, Context, ContextSelectionSet, Result};
use graphql_parser::query::{Field, Value};
use std::borrow::Cow;

pub trait GQLType {
    fn type_name() -> Cow<'static, str>;

    fn qualified_type_name() -> String {
        format!("{}!", Self::type_name())
    }

    fn create_type_info(registry: &mut registry::Registry) -> String;
}

pub trait GQLInputValue: GQLType + Sized {
    fn parse(value: &Value) -> Option<Self>;
}

#[async_trait::async_trait]
pub trait GQLOutputValue: GQLType {
    async fn resolve(value: &Self, ctx: &ContextSelectionSet<'_>) -> Result<serde_json::Value>;
}

#[async_trait::async_trait]
pub trait GQLObject: GQLOutputValue {
    fn is_empty() -> bool {
        return false;
    }

    async fn resolve_field(&self, ctx: &Context<'_>, field: &Field) -> Result<serde_json::Value>;
}

pub trait GQLInputObject: GQLInputValue {}

pub trait GQLScalar: Sized + Send {
    fn type_name() -> &'static str;
    fn description() -> Option<&'static str> {
        None
    }
    fn parse(value: &Value) -> Option<Self>;
    fn to_json(&self) -> Result<serde_json::Value>;
}

#[macro_export]
macro_rules! impl_scalar {
    ($ty:ty) => {
        impl crate::GQLType for $ty {
            fn type_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(<$ty as crate::GQLScalar>::type_name())
            }

            fn create_type_info(registry: &mut crate::registry::Registry) -> String {
                registry.create_type::<$ty, _>(|_| crate::registry::Type::Scalar {
                    name: <$ty as crate::GQLScalar>::type_name().to_string(),
                    description: <$ty>::description(),
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
    };
}

#[async_trait::async_trait]
impl<T: GQLObject + Send + Sync> GQLOutputValue for T {
    async fn resolve(value: &Self, ctx: &ContextSelectionSet<'_>) -> Result<serde_json::Value> {
        crate::resolver::do_resolve(ctx, value).await
    }
}
