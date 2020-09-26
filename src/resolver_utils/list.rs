use crate::extensions::{ErrorLogger, Extension, ResolveInfo};
use crate::parser::types::Field;
use crate::{ContextSelectionSet, OutputValueType, Positioned, Result, Type};

/// Resolve an list by executing each of the items concurrently.
pub async fn resolve_list<'a, T: OutputValueType + Send + Sync + 'a>(
    ctx: &ContextSelectionSet<'a>,
    field: &Positioned<Field>,
    iter: impl IntoIterator<Item = T>,
) -> Result<serde_json::Value> {
    let mut futures = Vec::new();

    for (idx, item) in iter.into_iter().enumerate() {
        let ctx_idx = ctx.with_index(idx);
        futures.push(async move {
            let resolve_info = ResolveInfo {
                resolve_id: ctx_idx.resolve_id,
                path_node: ctx_idx.path_node.as_ref().unwrap(),
                parent_type: &Vec::<T>::type_name(),
                return_type: &T::qualified_type_name(),
                schema_env: ctx.schema_env,
                query_env: ctx.query_env,
            };

            ctx_idx
                .query_env
                .extensions
                .lock()
                .resolve_start(&resolve_info);

            let res = OutputValueType::resolve(&item, &ctx_idx, field)
                .await
                .log_error(&ctx_idx.query_env.extensions)?;

            ctx_idx
                .query_env
                .extensions
                .lock()
                .resolve_end(&resolve_info);

            Result::Ok(res)
        });
    }

    Ok(futures::future::try_join_all(futures).await?.into())
}
