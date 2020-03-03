use crate::{schema, ContextSelectionSet, Result};
use graphql_parser::query::Value;
use std::borrow::Cow;

#[doc(hidden)]
pub trait GQLType {
    fn type_name() -> Cow<'static, str>;

    fn create_type_info(registry: &mut schema::Registry) -> String;
}

#[doc(hidden)]
pub trait GQLInputValue: GQLType + Sized {
    fn parse(value: Value) -> Result<Self>;
    fn parse_from_json(value: serde_json::Value) -> Result<Self>;
}

#[doc(hidden)]
#[async_trait::async_trait]
pub trait GQLOutputValue: GQLType {
    async fn resolve(&self, ctx: &ContextSelectionSet<'_>) -> Result<serde_json::Value>;
}

#[doc(hidden)]
pub trait GQLObject: GQLOutputValue {}

#[doc(hidden)]
pub trait GQLInputObject: GQLInputValue {}

pub trait Scalar: Sized + Send {
    fn type_name() -> &'static str;
    fn parse(value: Value) -> Result<Self>;
    fn parse_from_json(value: serde_json::Value) -> Result<Self>;
    fn to_json(&self) -> Result<serde_json::Value>;
}

impl<T: Scalar> GQLType for T {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed(T::type_name())
    }

    fn create_type_info(registry: &mut schema::Registry) -> String {
        registry.create_type(T::type_name(), |_| schema::Type::Scalar)
    }
}

impl<T: Scalar> GQLInputValue for T {
    fn parse(value: Value) -> Result<Self> {
        T::parse(value)
    }

    fn parse_from_json(value: serde_json::Value) -> Result<Self> {
        T::parse_from_json(value)
    }
}

#[async_trait::async_trait]
impl<T: Scalar + Sync> GQLOutputValue for T {
    async fn resolve(&self, _: &ContextSelectionSet<'_>) -> Result<serde_json::Value> {
        T::to_json(self)
    }
}
