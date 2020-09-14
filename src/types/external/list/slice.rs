use crate::parser::types::Field;
use crate::{registry, ContextSelectionSet, OutputValueType, Positioned, Result, Type};
use std::borrow::Cow;

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
