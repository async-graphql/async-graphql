use crate::{registry, ContextSelectionSet, GQLInputValue, GQLOutputValue, GQLType, Result, Value};
use std::borrow::Cow;

impl<T: GQLType> GQLType for Option<T> {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn qualified_type_name() -> String {
        T::type_name().to_string()
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        T::create_type_info(registry)
    }
}

impl<T: GQLInputValue> GQLInputValue for Option<T> {
    fn parse(value: &Value) -> Option<Self> {
        match value {
            Value::Null => Some(None),
            _ => Some(GQLInputValue::parse(value)?),
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
