use crate::base::BoxFieldFuture;
use crate::{ContextSelectionSet, Error, ErrorWithPosition, ObjectType, QueryError, Result};
use futures::{future, TryFutureExt};
use graphql_parser::query::{Selection, TypeCondition};
use std::iter::FromIterator;

#[allow(missing_docs)]
pub async fn do_resolve<'a, T: ObjectType + Send + Sync>(
    ctx: &'a ContextSelectionSet<'a>,
    root: &'a T,
) -> Result<serde_json::Value> {
    let mut futures = Vec::new();
    collect_fields(ctx.clone(), root, &mut futures)?;
    let res = futures::future::try_join_all(futures).await?;
    let map = serde_json::Map::from_iter(res);
    Ok(map.into())
}

#[allow(missing_docs)]
pub fn collect_fields<'a, T: ObjectType + Send + Sync>(
    ctx: ContextSelectionSet<'a>,
    root: &'a T,
    futures: &mut Vec<BoxFieldFuture<'a>>,
) -> Result<()> {
    if ctx.items.is_empty() {
        anyhow::bail!(QueryError::MustHaveSubFields {
            object: T::type_name().to_string(),
        }
        .with_position(ctx.span.0));
    }

    for selection in &ctx.item.items {
        match selection {
            Selection::Field(field) => {
                if ctx.is_skip(&field.directives)? {
                    continue;
                }

                let ctx_field = ctx.with_item(field);
                let field_name = ctx_field.result_name();

                if field.name.as_str() == "__typename" {
                    // Get the typename
                    futures.push(Box::pin(
                        future::ok::<serde_json::Value, Error>(T::type_name().to_string().into())
                            .map_ok(move |value| (field_name, value)),
                    ));
                    continue;
                }

                futures.push(Box::pin({
                    let ctx_field = ctx_field.clone();
                    async move {
                        root.resolve_field(&ctx_field, field)
                            .map_ok(move |value| (field_name, value))
                            .await
                    }
                }))
            }
            Selection::FragmentSpread(fragment_spread) => {
                if ctx.is_skip(&fragment_spread.directives)? {
                    continue;
                }

                if let Some(fragment) = ctx.fragments.get(&fragment_spread.fragment_name) {
                    collect_fields(ctx.with_item(&fragment.selection_set), root, futures)?;
                } else {
                    return Err(QueryError::UnknownFragment {
                        name: fragment_spread.fragment_name.clone(),
                    }
                    .into());
                }
            }
            Selection::InlineFragment(inline_fragment) => {
                if ctx.is_skip(&inline_fragment.directives)? {
                    continue;
                }

                if let Some(TypeCondition::On(name)) = &inline_fragment.type_condition {
                    root.collect_inline_fields(
                        name,
                        ctx.with_item(&inline_fragment.selection_set),
                        futures,
                    )?;
                }
            }
        }
    }

    Ok(())
}
