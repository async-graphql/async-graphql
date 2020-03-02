use crate::{ContextSelectionSet, Result};
use graphql_parser::query::Value;
use std::borrow::Cow;

#[doc(hidden)]
pub trait GQLType {
    fn type_name() -> Cow<'static, str>;
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
