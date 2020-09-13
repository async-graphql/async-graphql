use crate::parser::types::Field;
use crate::{
    registry, ContextSelectionSet, InputValueResult, InputValueType, OutputValueType, Positioned,
    Result, Type, Value,
};
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
    fn parse(value: Option<Value>) -> InputValueResult<Self> {
        match value.unwrap_or_default() {
            Value::List(values) => {
                let mut result = Vec::new();
                for elem_value in values {
                    result.push(InputValueType::parse(Some(elem_value))?);
                }
                Ok(result)
            }
            value => Ok(vec![InputValueType::parse(Some(value))?]),
        }
    }

    fn to_value(&self) -> Value {
        Value::List(self.iter().map(InputValueType::to_value).collect())
    }
}

#[allow(clippy::ptr_arg)]
#[async_trait::async_trait]
impl<T: OutputValueType + Send + Sync> OutputValueType for Vec<T> {
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> Result<serde_json::Value> {
        let mut futures = Vec::with_capacity(self.len());
        for (idx, item) in self.iter().enumerate() {
            let ctx_idx = ctx.with_index(idx);
            futures.push(async move { OutputValueType::resolve(item, &ctx_idx, field).await });
        }
        Ok(futures::future::try_join_all(futures).await?.into())
    }
}

impl<'a, T: Type + 'a> Type for &'a [T] {
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

#[async_trait::async_trait]
impl<T: OutputValueType + Send + Sync> OutputValueType for &[T] {
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> Result<serde_json::Value> {
        let mut futures = Vec::with_capacity(self.len());
        for (idx, item) in (*self).iter().enumerate() {
            let ctx_idx = ctx.with_index(idx);
            futures.push(async move { OutputValueType::resolve(item, &ctx_idx, field).await });
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

        assert_eq!(<&[i32] as Type>::qualified_type_name(), "[Int!]!");
    }
}
