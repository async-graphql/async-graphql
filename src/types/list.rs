use crate::{registry, ContextSelectionSet, InputValueType, OutputValueType, Result, Type, Value};
use std::borrow::Cow;

impl<T: Type> Type for Vec<T> {
    fn type_name() -> Cow<'static, str> {
        Cow::Owned(format!("[{}]", T::qualified_type_name()))
    }

    fn qualified_type_name() -> String {
        format!("[{}]!", T::qualified_type_name())
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        T::create_type_info(registry);
        Self::qualified_type_name()
    }
}

impl<T: InputValueType> InputValueType for Vec<T> {
    fn parse(value: &Value) -> Option<Self> {
        match value {
            Value::List(values) => {
                let mut result = Vec::new();
                for value in values {
                    result.push(InputValueType::parse(value)?);
                }
                Some(result)
            }
            _ => None,
        }
    }
}

#[allow(clippy::ptr_arg)]
#[async_trait::async_trait]
impl<T: OutputValueType + Send + Sync> OutputValueType for Vec<T> {
    async fn resolve(value: &Self, ctx: &ContextSelectionSet<'_>) -> Result<serde_json::Value> {
        let mut futures = Vec::with_capacity(value.len());
        for item in value {
            futures.push(OutputValueType::resolve(item, &ctx));
        }
        Ok(futures::future::try_join_all(futures).await?.into())
    }
}

impl<T: Type> Type for &[T] {
    fn type_name() -> Cow<'static, str> {
        Cow::Owned(format!("[{}]", T::type_name()))
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        T::create_type_info(registry)
    }
}

#[async_trait::async_trait]
impl<T: OutputValueType + Send + Sync> OutputValueType for &[T] {
    async fn resolve(value: &Self, ctx: &ContextSelectionSet<'_>) -> Result<serde_json::Value> {
        let mut futures = Vec::with_capacity(value.len());
        for item in *value {
            futures.push(OutputValueType::resolve(item, &ctx));
        }
        Ok(futures::future::try_join_all(futures).await?.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::Type;

    #[test]
    fn test_list_type() {
        assert_eq!(Vec::<i32>::type_name(), "[Int!]");
        assert_eq!(Vec::<Option<i32>>::type_name(), "[Int]");
        assert_eq!(Option::<Vec::<Option<i32>>>::type_name(), "[Int]");

        assert_eq!(Vec::<i32>::qualified_type_name(), "[Int!]!");
        assert_eq!(Vec::<Option<i32>>::qualified_type_name(), "[Int]!");
        assert_eq!(Option::<Vec::<Option<i32>>>::qualified_type_name(), "[Int]");
    }
}
