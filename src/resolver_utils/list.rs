use crate::{
    ContextSelectionSet, OutputType, OutputTypeMarker, Positioned, ServerResult, Value,
    extensions::ResolveInfo, parser::types::Field,
};
#[cfg(feature = "boxed-trait")]
/// Resolve an list by executing each of the items concurrently.
pub async fn resolve_list<'a, T: OutputTypeMarker>(
    ctx: &ContextSelectionSet<'a>,
    field: &Positioned<Field>,
    iter: impl IntoIterator<Item = &dyn OutputType>,
    len: Option<usize>,
) -> ServerResult<Value> {
    let type_name = T::type_name();
    let qualified_type_name = T::qualified_type_name();

    async fn resolve_list_inner<'a>(
        ctx: &ContextSelectionSet<'a>,
        field: &Positioned<Field>,
        iter: impl IntoIterator<Item = &dyn OutputType>,
        len: Option<usize>,
        type_name: std::borrow::Cow<'a, str>,
        qualified_type_name: String,
    ) -> Result<Value, crate::ServerError> {
        let extensions = &ctx.query_env.extensions;
        if !extensions.is_empty() {
            let mut futures = len.map(Vec::with_capacity).unwrap_or_default();
            for (idx, item) in iter.into_iter().enumerate() {
                futures.push({
                    let ctx = ctx.clone();
                    let type_name = type_name.clone();
                    let qualified_type_name = qualified_type_name.clone();
                    async move {
                        let ctx_idx = ctx.with_index(idx);
                        let extensions = &ctx.query_env.extensions;

                        let resolve_info = ResolveInfo {
                            path_node: ctx_idx.path_node.as_ref().unwrap(),
                            parent_type: &type_name,
                            return_type: &qualified_type_name,
                            name: field.node.name.node.as_str(),
                            alias: field.node.alias.as_ref().map(|alias| alias.node.as_str()),
                            is_for_introspection: ctx_idx.is_for_introspection,
                            field: &field.node,
                        };
                        let resolve_fut = async {
                            OutputType::resolve(&item, &ctx_idx, field)
                                .await
                                .map(Option::Some)
                                .map_err(|err| ctx_idx.set_error_path(err))
                        };
                        futures_util::pin_mut!(resolve_fut);
                        extensions
                            .resolve(resolve_info, &mut resolve_fut)
                            .await
                            .map(|value| value.expect("You definitely encountered a bug!"))
                    }
                });
            }
            Ok(Value::List(
                futures_util::future::try_join_all(futures).await?,
            ))
        } else {
            let mut futures = len.map(Vec::with_capacity).unwrap_or_default();
            for (idx, item) in iter.into_iter().enumerate() {
                let ctx_idx = ctx.with_index(idx);
                futures.push(async move {
                    OutputType::resolve(&item, &ctx_idx, field)
                        .await
                        .map_err(|err| ctx_idx.set_error_path(err))
                });
            }
            Ok(Value::List(
                futures_util::future::try_join_all(futures).await?,
            ))
        }
    }

    resolve_list_inner(ctx, field, iter, len, type_name, qualified_type_name).await
}

#[cfg(not(feature = "boxed-trait"))]
/// Resolve an list by executing each of the items concurrently.
pub async fn resolve_list<'a, T: OutputType + OutputTypeMarker + 'a>(
    ctx: &ContextSelectionSet<'a>,
    field: &Positioned<Field>,
    iter: impl IntoIterator<Item = T>,
    len: Option<usize>,
) -> ServerResult<Value> {
    let extensions = &ctx.query_env.extensions;
    if !extensions.is_empty() {
        let mut futures = len.map(Vec::with_capacity).unwrap_or_default();
        for (idx, item) in iter.into_iter().enumerate() {
            futures.push({
                let ctx = ctx.clone();
                async move {
                    let ctx_idx = ctx.with_index(idx);
                    let extensions = &ctx.query_env.extensions;

                    let resolve_info = ResolveInfo {
                        path_node: ctx_idx.path_node.as_ref().unwrap(),
                        parent_type: &Vec::<T>::type_name(),
                        return_type: &T::qualified_type_name(),
                        name: field.node.name.node.as_str(),
                        alias: field.node.alias.as_ref().map(|alias| alias.node.as_str()),
                        is_for_introspection: ctx_idx.is_for_introspection,
                        field: &field.node,
                    };
                    let resolve_fut = async {
                        OutputType::resolve(&item, &ctx_idx, field)
                            .await
                            .map(Option::Some)
                            .map_err(|err| ctx_idx.set_error_path(err))
                    };
                    futures_util::pin_mut!(resolve_fut);
                    extensions
                        .resolve(resolve_info, &mut resolve_fut)
                        .await
                        .map(|value| value.expect("You definitely encountered a bug!"))
                }
            });
        }
        Ok(Value::List(
            futures_util::future::try_join_all(futures).await?,
        ))
    } else {
        let mut futures = len.map(Vec::with_capacity).unwrap_or_default();
        for (idx, item) in iter.into_iter().enumerate() {
            let ctx_idx = ctx.with_index(idx);
            futures.push(async move {
                OutputType::resolve(&item, &ctx_idx, field)
                    .await
                    .map_err(|err| ctx_idx.set_error_path(err))
            });
        }
        Ok(Value::List(
            futures_util::future::try_join_all(futures).await?,
        ))
    }
}
