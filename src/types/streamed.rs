use crate::context::DeferList;
use crate::registry::Registry;
use crate::{ContextSelectionSet, OutputValueType, Positioned, QueryResponse, Result, Type};
use async_graphql_parser::query::Field;
use itertools::Itertools;
use parking_lot::Mutex;
use std::borrow::Cow;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

/// Streamed type
///
/// Similar to Deferred, but you can defer every item of the list type, only takes effect when the @stream directive exists on the field.
pub struct Streamed<T: Type + Send + Sync + 'static>(Mutex<Option<Vec<T>>>);

impl<T: Type + Send + Sync + 'static> From<Vec<T>> for Streamed<T> {
    fn from(value: Vec<T>) -> Self {
        Self(Mutex::new(Some(value)))
    }
}

impl<T: Type + Send + Sync + 'static> Type for Streamed<T> {
    fn type_name() -> Cow<'static, str> {
        Vec::<T>::type_name()
    }

    fn create_type_info(registry: &mut Registry) -> String {
        Vec::<T>::create_type_info(registry)
    }
}

#[async_trait::async_trait]
impl<T: OutputValueType + Send + Sync + 'static> OutputValueType for Streamed<T> {
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> Result<serde_json::Value> {
        let list = self.0.lock().take();
        if let Some(list) = list {
            if let Some(defer_list) = ctx.defer_list {
                if ctx.is_stream(&field.directives) {
                    let mut field = field.clone();

                    // remove @stream directive
                    if let Some((idx, _)) = field
                        .node
                        .directives
                        .iter()
                        .find_position(|d| d.name.as_str() == "stream")
                    {
                        field.node.directives.remove(idx);
                    }

                    let field = Arc::new(field);

                    let path_prefix = ctx
                        .path_node
                        .as_ref()
                        .map(|path| path.to_json())
                        .unwrap_or_default();

                    for (idx, item) in list.into_iter().enumerate() {
                        let path_prefix = {
                            let mut path_prefix = path_prefix.clone();
                            path_prefix.push(serde_json::Value::Number(idx.into()));
                            path_prefix
                        };
                        let field = field.clone();
                        let schema_env = ctx.schema_env.clone();
                        let query_env = ctx.query_env.clone();

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
                            let data = item.resolve(&ctx, &field).await?;

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
                    }
                    return Ok(serde_json::Value::Array(Vec::new()));
                }
            }
            OutputValueType::resolve(&list, ctx, field).await
        } else {
            Ok(serde_json::Value::Null)
        }
    }
}
