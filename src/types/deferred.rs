use crate::context::DeferList;
use crate::registry::Registry;
use crate::{ContextSelectionSet, OutputValueType, Pos, QueryResponse, Result, Type};
use std::borrow::Cow;
use std::sync::atomic::AtomicUsize;

pub struct Deferred<T: Type + Send + Sync + Clone + 'static>(T);

impl<T: Type + Send + Sync + Clone + 'static> From<T> for Deferred<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: Type + Send + Sync + Clone + 'static> Type for Deferred<T> {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut Registry) -> String {
        T::create_type_info(registry)
    }
}

#[async_trait::async_trait]
impl<T: OutputValueType + Send + Sync + Clone + 'static> OutputValueType for Deferred<T> {
    async fn resolve(&self, ctx: &ContextSelectionSet<'_>, pos: Pos) -> Result<serde_json::Value> {
        if let Some(defer_list) = ctx.defer_list {
            let obj = self.0.clone();
            let schema_env = ctx.schema_env.clone();
            let query_env = ctx.query_env.clone();
            let field_selection_set = ctx.item.clone();
            let path = ctx.path_node.as_ref().map(|path| path.to_json());
            defer_list.append(async move {
                let inc_resolve_id = AtomicUsize::default();
                let defer_list = DeferList::default();
                let ctx = query_env.create_context(
                    &schema_env,
                    None,
                    &field_selection_set,
                    &inc_resolve_id,
                    Some(&defer_list),
                );
                let data = obj.resolve(&ctx, pos).await?;

                Ok((
                    QueryResponse {
                        path,
                        data,
                        extensions: None,
                        cache_control: Default::default(),
                    },
                    defer_list,
                ))
            });
            Ok(serde_json::Value::Null)
        } else {
            OutputValueType::resolve(&self.0, ctx, pos).await
        }
    }
}
