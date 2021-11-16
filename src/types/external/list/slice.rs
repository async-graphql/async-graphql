use std::borrow::Cow;

use crate::parser::types::Field;
use crate::resolver_utils::resolve_list;
use crate::{registry, ContextSelectionSet, OutputType, Positioned, ServerResult, Value};

#[async_trait::async_trait]
impl<'a, T: OutputType + 'a> OutputType for &'a [T] {
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

    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        resolve_list(ctx, field, self.iter(), Some(self.len())).await
    }
}
