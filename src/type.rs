use crate::{ContextSelectionSet, ErrorWithPosition, QueryError, Result};
use graphql_parser::query::Value;

#[doc(hidden)]
pub trait GQLType {
    fn type_name() -> String;
}

#[doc(hidden)]
pub trait GQLInputValue: GQLType + Sized {
    fn parse(value: Value) -> Result<Self>;
}

#[doc(hidden)]
#[async_trait::async_trait]
pub trait GQLOutputValue: GQLType {
    async fn resolve(self, ctx: &ContextSelectionSet<'_>) -> Result<serde_json::Value>;
}

impl<T: GQLType> GQLType for Vec<T> {
    fn type_name() -> String {
        format!("[{}]!", T::type_name()).into()
    }
}

impl<T: GQLInputValue> GQLInputValue for Vec<T> {
    fn parse(value: Value) -> Result<Self> {
        match value {
            Value::List(values) => {
                let mut result = Vec::new();
                for value in values {
                    result.push(GQLInputValue::parse(value)?);
                }
                Ok(result)
            }
            _ => {
                return Err(QueryError::ExpectedType {
                    expect: Self::type_name(),
                    actual: value,
                }
                .into())
            }
        }
    }
}

#[async_trait::async_trait]
impl<T: GQLOutputValue + Send> GQLOutputValue for Vec<T> {
    async fn resolve(self, ctx: &ContextSelectionSet<'_>) -> Result<serde_json::Value> {
        let mut res = Vec::new();
        for item in self {
            res.push(item.resolve(ctx).await?);
        }
        Ok(res.into())
    }
}

impl<T: GQLType> GQLType for Option<T> {
    fn type_name() -> String {
        format!("{}", T::type_name().trim_end_matches("!")).into()
    }
}

impl<T: GQLInputValue> GQLInputValue for Option<T> {
    fn parse(value: Value) -> Result<Self> {
        match value {
            Value::Null => Ok(None),
            _ => Ok(Some(GQLInputValue::parse(value)?)),
        }
    }
}

#[async_trait::async_trait]
impl<T: GQLOutputValue + Send> GQLOutputValue for Option<T> {
    async fn resolve(self, ctx: &ContextSelectionSet<'_>) -> Result<serde_json::Value> {
        if let Some(inner) = self {
            inner.resolve(ctx).await
        } else {
            Ok(serde_json::Value::Null)
        }
    }
}

#[doc(hidden)]
pub trait GQLObject: GQLOutputValue {}

pub struct GQLEmptyMutation;

impl GQLType for GQLEmptyMutation {
    fn type_name() -> String {
        "EmptyMutation".to_string()
    }
}

#[async_trait::async_trait]
impl GQLOutputValue for GQLEmptyMutation {
    async fn resolve(self, ctx: &ContextSelectionSet<'_>) -> Result<serde_json::Value> {
        anyhow::bail!(QueryError::NotConfiguredMutations.with_position(ctx.item.span.0));
    }
}

impl GQLObject for GQLEmptyMutation {}

#[doc(hidden)]
pub trait GQLInputObject: GQLInputValue {}
