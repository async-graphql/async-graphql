use crate::parser::types::Field;
use crate::resolver_utils::resolve_list;
use crate::{registry, ContextSelectionSet, OutputType, Positioned, ServerResult, Type, Value};

impl<'a, T: Type + 'a> Type for &'a [T] {
    fn type_name() -> &'static str {
        static NAME: once_cell::sync::OnceCell<String> = once_cell::sync::OnceCell::new();
        NAME.get_or_init(|| format!("[{}]", T::qualified_type_name()))
            .as_str()
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
impl<T: OutputType> OutputType for &[T] {
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        resolve_list(ctx, field, self.iter(), Some(self.len())).await
    }
}
