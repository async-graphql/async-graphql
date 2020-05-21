use crate::context::DeferList;
use crate::registry::Registry;
use crate::{ContextSelectionSet, OutputValueType, Positioned, QueryResponse, Result, Type};
use async_graphql_parser::query::Field;
use itertools::Itertools;
use parking_lot::Mutex;
use std::borrow::Cow;
use std::sync::atomic::AtomicUsize;

/// Deferred type
///
/// Allows to defer the type of results returned, only takes effect when the @defer directive exists on the field.
pub struct Deferred<T: Type + Send + Sync + 'static>(Mutex<Option<T>>);

impl<T: Type + Send + Sync + 'static> From<T> for Deferred<T> {
    fn from(value: T) -> Self {
        Self(Mutex::new(Some(value)))
    }
}

impl<T: Type + Send + Sync + 'static> Type for Deferred<T> {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut Registry) -> String {
        T::create_type_info(registry)
    }
}

#[async_trait::async_trait]
impl<T: OutputValueType + Send + Sync + 'static> OutputValueType for Deferred<T> {
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> Result<serde_json::Value> {
        let obj = self.0.lock().take();
        if let Some(obj) = obj {
            if let Some(defer_list) = ctx.defer_list {
                if ctx.is_defer(&field.directives) {
                    let schema_env = ctx.schema_env.clone();
                    let query_env = ctx.query_env.clone();
                    let mut field = field.clone();

                    // remove @defer directive
                    if let Some((idx, _)) = field
                        .node
                        .directives
                        .iter()
                        .find_position(|d| d.name.as_str() == "defer")
                    {
                        field.node.directives.remove(idx);
                    }

                    let path_prefix = ctx
                        .path_node
                        .as_ref()
                        .map(|path| path.to_json())
                        .unwrap_or_default();

                    defer_list.append(async move {
                        let inc_resolve_id = AtomicUsize::default();
                        let defer_list = DeferList {
                            path_prefix: path_prefix.clone(),
                            futures: Default::default(),
                        };
                        let ctx = query_env.create_context(
                            &schema_env,
                            None,
                            &field.selection_set,
                            &inc_resolve_id,
                            Some(&defer_list),
                        );
                        let data = obj.resolve(&ctx, &field).await?;

                        Ok((
                            QueryResponse {
                                path: Some(path_prefix),
                                data,
                                extensions: None,
                                cache_control: Default::default(),
                            },
                            defer_list,
                        ))
                    });
                    return Ok(serde_json::Value::Null);
                }
            }
            OutputValueType::resolve(&obj, ctx, field).await
        } else {
            Ok(serde_json::Value::Null)
        }
    }
}
