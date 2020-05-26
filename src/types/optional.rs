use crate::{
    registry, ContextSelectionSet, InputValueResult, InputValueType, OutputValueType, Positioned,
    Result, Type, Value,
};
use async_graphql_parser::query::Field;
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
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Null => Ok(None),
            _ => Ok(Some(T::parse(value)?)),
        }
    }

    fn to_value(&self) -> Value {
        match self {
            Some(value) => value.to_value(),
            None => Value::Null,
        }
    }
}

#[async_trait::async_trait]
impl<T: OutputValueType + Sync> OutputValueType for Option<T> {
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> Result<serde_json::Value> where {
        if let Some(inner) = self {
            OutputValueType::resolve(inner, ctx, field).await
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
