use crate::base::BoxFieldFuture;
use crate::extensions::ResolveInfo;
use crate::{ContextSelectionSet, Error, ObjectType, QueryError, Result};
use futures::{future, TryFutureExt};
use graphql_parser::query::{Selection, TypeCondition};
use std::iter::FromIterator;

#[allow(missing_docs)]
pub async fn do_resolve<'a, T: ObjectType + Send + Sync>(
    ctx: &'a ContextSelectionSet<'a>,
    root: &'a T,
) -> Result<serde_json::Value> {
    let mut futures = Vec::new();
    collect_fields(ctx, root, &mut futures)?;
    let res = futures::future::try_join_all(futures).await?;
    let map = serde_json::Map::from_iter(res);
    Ok(map.into())
}

#[allow(missing_docs)]
pub fn collect_fields<'a, T: ObjectType + Send + Sync>(
    ctx: &ContextSelectionSet<'a>,
    root: &'a T,
    futures: &mut Vec<BoxFieldFuture<'a>>,
) -> Result<()> {
    if ctx.items.is_empty() {
        return Err(Error::Query {
            pos: ctx.span.0,
            path: None,
            err: QueryError::MustHaveSubFields {
                object: T::type_name().to_string(),
            },
        });
    }

    for selection in &ctx.item.items {
        match selection {
            Selection::Field(field) => {
                if ctx.is_skip(&field.directives)? {
                    continue;
                }

                if field.name.as_str() == "__typename" {
                    // Get the typename
                    let ctx_field = ctx.with_field(field);
                    let field_name = ctx_field.result_name().to_string();
                    futures.push(Box::pin(
                        future::ok::<serde_json::Value, Error>(
                            root.introspection_type_name().to_string().into(),
                        )
                        .map_ok(move |value| (field_name, value)),
                    ));
                    continue;
                }

                futures.push(Box::pin({
                    let ctx = ctx.clone();
                    async move {
                        let ctx_field = ctx.with_field(field);
                        let field_name = ctx_field.result_name().to_string();
                        let resolve_id = ctx_field.get_resolve_id();

                        if !ctx_field.extensions.is_empty() {
                            let resolve_info = ResolveInfo {
                                resolve_id,
                                path_node: ctx_field.path_node.as_ref().unwrap(),
                                parent_type: &T::type_name(),
                                return_type: match ctx_field
                                    .registry
                                    .types
                                    .get(T::type_name().as_ref())
                                    .and_then(|ty| ty.field_by_name(field.name.as_str()))
                                    .map(|field| &field.ty)
                                {
                                    Some(ty) => &ty,
                                    None => {
                                        return Err(Error::Query {
                                            pos: field.position,
                                            path: None,
                                            err: QueryError::FieldNotFound {
                                                field_name: field.name.clone(),
                                                object: T::type_name().to_string(),
                                            },
                                        });
                                    }
                                },
                            };

                            ctx_field
                                .extensions
                                .iter()
                                .for_each(|e| e.resolve_field_start(&resolve_info));
                        }

                        let res = root
                            .resolve_field(&ctx_field, field)
                            .map_ok(move |value| (field_name, value))
                            .await?;

                        if !ctx_field.extensions.is_empty() {
                            ctx_field
                                .extensions
                                .iter()
                                .for_each(|e| e.resolve_field_end(resolve_id));
                        }

                        Ok(res)
                    }
                }))
            }
            Selection::FragmentSpread(fragment_spread) => {
                if ctx.is_skip(&fragment_spread.directives)? {
                    continue;
                }

                if let Some(fragment) = ctx.fragments.get(fragment_spread.fragment_name.as_str()) {
                    collect_fields(
                        &ctx.with_selection_set(&fragment.selection_set),
                        root,
                        futures,
                    )?;
                } else {
                    return Err(Error::Query {
                        pos: fragment_spread.position,
                        path: None,
                        err: QueryError::UnknownFragment {
                            name: fragment_spread.fragment_name.clone(),
                        },
                    });
                }
            }
            Selection::InlineFragment(inline_fragment) => {
                if ctx.is_skip(&inline_fragment.directives)? {
                    continue;
                }

                if let Some(TypeCondition::On(name)) = &inline_fragment.type_condition {
                    root.collect_inline_fields(
                        name,
                        inline_fragment.position,
                        &ctx.with_selection_set(&inline_fragment.selection_set),
                        futures,
                    )?;
                } else {
                    collect_fields(
                        &ctx.with_selection_set(&inline_fragment.selection_set),
                        root,
                        futures,
                    )?;
                }
            }
        }
    }

    Ok(())
}
