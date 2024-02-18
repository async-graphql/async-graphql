use std::borrow::Cow;

use async_graphql_parser::types::Field;
use tokio::sync::Mutex;

use crate::{registry, ContextSelectionSet, OutputType, Positioned, ServerResult, Value};

impl<T: OutputType> OutputType for Mutex<T> {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        <T as OutputType>::create_type_info(registry)
    }

    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        self.lock().await.resolve(ctx, field).await
    }
}
