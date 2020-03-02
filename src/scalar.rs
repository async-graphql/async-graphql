use crate::r#type::{GQLInputValue, GQLOutputValue, GQLType};
use crate::{ContextSelectionSet, Result};
use graphql_parser::query::Value;
use std::borrow::Cow;

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
