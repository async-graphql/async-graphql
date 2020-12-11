use crate::extensions::{ErrorLogger, ExtensionContext, ResolveInfo};
use crate::parser::types::Field;
use crate::{ContextSelectionSet, OutputType, PathSegment, Positioned, ServerResult, Type, Value};

/// Resolve an list by executing each of the items concurrently.
pub async fn resolve_list<'a, T: OutputType + Send + Sync + 'a>(
    ctx: &ContextSelectionSet<'a>,
    field: &Positioned<Field>,
    iter: impl IntoIterator<Item = T>,
    len: Option<usize>,
) -> ServerResult<Value> {
    let mut futures = len.map(Vec::with_capacity).unwrap_or_default();

    for (idx, item) in iter.into_iter().enumerate() {
        let ctx_idx = ctx.with_index(idx);
        futures.push(async move {
            let ctx_extension = ExtensionContext {
                schema_data: &ctx.schema_env.data,
                query_data: &ctx.query_env.ctx_data,
            };

            if ctx_idx.query_env.extensions.is_empty() {
                OutputType::resolve(&item, &ctx_idx, field)
                    .await
                    .map_err(|e| e.path(PathSegment::Index(idx)))
                    .log_error(&ctx_extension, &ctx_idx.query_env.extensions)
            } else {
                let resolve_info = ResolveInfo {
                    resolve_id: ctx_idx.resolve_id,
                    path_node: ctx_idx.path_node.as_ref().unwrap(),
                    parent_type: &Vec::<T>::type_name(),
                    return_type: &T::qualified_type_name(),
                };

                ctx_idx
                    .query_env
                    .extensions
                    .resolve_start(&ctx_extension, &resolve_info);

                let res = OutputType::resolve(&item, &ctx_idx, field)
                    .await
                    .map_err(|e| e.path(PathSegment::Index(idx)))
                    .log_error(&ctx_extension, &ctx_idx.query_env.extensions)?;

                ctx_idx
                    .query_env
                    .extensions
                    .resolve_end(&ctx_extension, &resolve_info);

                Ok(res)
            }
        });
    }

    Ok(Value::List(
        futures_util::future::try_join_all(futures).await?,
    ))
}
