use crate::parser::types::Field;
use crate::resolver_utils::resolve_list;
use crate::{
    registry, ContextSelectionSet, InputValueResult, InputValueType, OutputValueType, Positioned,
    Result, Type, Value,
};
use std::borrow::Cow;
use std::collections::VecDeque;

impl<T: Type> Type for VecDeque<T> {
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

impl<T: InputValueType> InputValueType for VecDeque<T> {
    fn parse(value: Option<Value>) -> InputValueResult<Self> {
        match value.unwrap_or_default() {
            Value::List(values) => {
                let mut result = Self::default();
                for elem_value in values {
                    result.extend(std::iter::once(InputValueType::parse(Some(elem_value))?));
                }
                Ok(result)
            }
            value => Ok({
                let mut result = Self::default();
                result.extend(std::iter::once(InputValueType::parse(Some(value))?));
                result
            }),
        }
    }

    fn to_value(&self) -> Value {
        Value::List(self.iter().map(InputValueType::to_value).collect())
    }
}

#[async_trait::async_trait]
impl<T: OutputValueType + Send + Sync> OutputValueType for VecDeque<T> {
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> Result<serde_json::Value> {
        resolve_list(ctx, field, self).await
    }
}
