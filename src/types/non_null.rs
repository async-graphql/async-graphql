use crate::{ContextSelectionSet, GQLInputValue, GQLOutputValue, GQLType, Result, Value};
use std::borrow::Cow;

impl<T: GQLType> GQLType for Option<T> {
    fn type_name() -> Cow<'static, str> {
        Cow::Owned(format!("{}", T::type_name().trim_end_matches("!")))
    }
}

impl<T: GQLInputValue> GQLInputValue for Option<T> {
    fn parse(value: Value) -> Result<Self> {
        match value {
            Value::Null => Ok(None),
            _ => Ok(Some(GQLInputValue::parse(value)?)),
        }
    }

    fn parse_from_json(value: serde_json::Value) -> Result<Self> {
        match value {
            serde_json::Value::Null => Ok(None),
            _ => Ok(Some(GQLInputValue::parse_from_json(value)?)),
        }
    }
}

#[async_trait::async_trait]
impl<T: GQLOutputValue + Send + Sync> GQLOutputValue for Option<T> {
    async fn resolve(&self, ctx: &ContextSelectionSet<'_>) -> Result<serde_json::Value> {
        if let Some(inner) = self {
            inner.resolve(ctx).await
        } else {
            Ok(serde_json::Value::Null)
        }
    }
}
