use crate::{
    registry, ContextSelectionSet, InputValueType, OutputValueType, Pos, Result, Type, Value,
};
use std::borrow::Cow;

impl<T: Type> Type for Option<T> {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn qualified_type_name() -> String {
        T::type_name().to_string()
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        T::create_type_info(registry);
        T::type_name().to_string()
    }
}

impl<T: InputValueType> InputValueType for Option<T> {
    fn parse(value: &Value) -> Option<Self> {
        match value {
            Value::Null => Some(None),
            _ => Some(Some(T::parse(value)?)),
        }
    }
}

#[async_trait::async_trait]
impl<T: OutputValueType + Sync> OutputValueType for Option<T> {
    async fn resolve(
        value: &Self,
        ctx: &ContextSelectionSet<'_>,
        pos: Pos,
    ) -> Result<serde_json::Value> where {
        if let Some(inner) = value {
            OutputValueType::resolve(inner, ctx, pos).await
        } else {
            Ok(serde_json::Value::Null)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Type;

    #[test]
    fn test_optional_type() {
        assert_eq!(Option::<i32>::type_name(), "Int");
        assert_eq!(Option::<i32>::qualified_type_name(), "Int");
        assert_eq!(&Option::<i32>::type_name(), "Int");
        assert_eq!(&Option::<i32>::qualified_type_name(), "Int");
    }
}
