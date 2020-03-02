use crate::{ContextSelectionSet, GQLInputValue, GQLOutputValue, GQLType, Result, Value};
use std::borrow::Cow;

impl<T: GQLType> GQLType for Option<T> {
    fn type_name() -> Cow<'static, str> {
        let name = T::type_name();
        Cow::Owned(format!("{}", &name[..name.len() - 1]))
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
impl<T: GQLOutputValue + Sync> GQLOutputValue for Option<T> {
    async fn resolve(&self, ctx: &ContextSelectionSet<'_>) -> Result<serde_json::Value> where {
        if let Some(inner) = self {
            inner.resolve(ctx).await
        } else {
            Ok(serde_json::Value::Null)
        }
    }
}