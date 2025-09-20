use std::borrow::Cow;

use async_graphql_parser::types::Field;
use tokio::sync::Mutex;

use crate::{ContextSelectionSet, OutputType, Positioned, ServerResult, Value, registry};

impl<T: OutputTypeMarker> OutputTypeMarker for Mutex<T> {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        <T as OutputType>::create_type_info(registry)
    }
}

#[cfg_attr(feature = "boxed-trait", async_trait::async_trait)]
impl<T: OutputType> OutputType for Mutex<T> {
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        self.lock().await.resolve(ctx, field).await
    }
}
