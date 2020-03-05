use crate::{registry, ContextSelectionSet, Result};
use graphql_parser::query::Value;
use std::borrow::Cow;

#[doc(hidden)]
pub trait GQLType {
    fn type_name() -> Cow<'static, str>;

    fn qualified_type_name() -> String {
        format!("{}!", Self::type_name())
    }

    fn create_type_info(registry: &mut registry::Registry) -> String;
}

#[doc(hidden)]
pub trait GQLInputValue: GQLType + Sized {
    fn parse(value: &Value) -> Option<Self>;
}

#[doc(hidden)]
#[async_trait::async_trait]
pub trait GQLOutputValue: GQLType {
    async fn resolve(&self, ctx: &ContextSelectionSet<'_>) -> Result<serde_json::Value>;
}

#[doc(hidden)]
pub trait GQLObject: GQLOutputValue {
    fn is_empty() -> bool {
        return false;
    }
}

#[doc(hidden)]
pub trait GQLInputObject: GQLInputValue {}

pub trait GQLScalar: Sized + Send {
    fn type_name() -> &'static str;
    fn description() -> Option<&'static str> {
        None
    }
    fn parse(value: &Value) -> Option<Self>;
    fn to_json(&self) -> Result<serde_json::Value>;
}

impl<T: GQLScalar> GQLType for T {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed(T::type_name())
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        registry.create_type::<T, _>(|_| registry::Type::Scalar {
            name: T::type_name().to_string(),
            description: T::description(),
        })
    }
}

impl<T: GQLScalar> GQLInputValue for T {
    fn parse(value: &Value) -> Option<Self> {
        T::parse(value)
    }
}

#[async_trait::async_trait]
impl<T: GQLScalar + Sync> GQLOutputValue for T {
    async fn resolve(&self, _: &ContextSelectionSet<'_>) -> Result<serde_json::Value> {
        T::to_json(self)
    }
}
