use crate::extensions::{ErrorLogger, Extension, ResolveInfo};
use crate::parser::types::{Selection, TypeCondition};
use crate::registry::MetaType;
use crate::{ContextSelectionSet, Error, ObjectType, QueryError, Result};
use std::future::Future;
use std::pin::Pin;

type BoxMutationFuture<'a> = Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;

#[allow(missing_docs)]
pub async fn do_mutation_resolve<'a, T: ObjectType + Send + Sync>(
    ctx: &'a ContextSelectionSet<'a>,
    root: &'a T,
) -> Result<serde_json::Value> {
    let mut values = serde_json::Map::new();
    do_resolve(ctx, root, &mut values).await?;
    Ok(values.into())
}

fn do_resolve<'a, T: ObjectType + Send + Sync>(
    ctx: &'a ContextSelectionSet<'a>,
    root: &'a T,
    values: &'a mut serde_json::Map<String, serde_json::Value>,
) -> BoxMutationFuture<'a> {
    Box::pin(async move {
        if ctx.item.node.items.is_empty() {
            return Err(Error::Query {
                pos: ctx.item.pos,
                path: None,
                err: QueryError::MustHaveSubFields {
                    object: T::type_name().to_string(),
                },
            });
        }

        for selection in &ctx.item.node.items {
            match &selection.node {
                Selection::Field(field) => {
                    if ctx.is_skip(&field.node.directives)? {
                        continue;
                    }

                    if field.node.name.node == "__typename" {
                        values.insert(
                            "__typename".to_string(),
                            root.introspection_type_name().to_string().into(),
                        );
                        continue;
                    }

                    if ctx.is_ifdef(&field.node.directives) {
                        if let Some(MetaType::Object { fields, .. }) =
                            ctx.schema_env.registry.types.get(T::type_name().as_ref())
                        {
                            if !fields.contains_key(field.node.name.node.as_str()) {
                                continue;
                            }
                        }
                    }

                    let ctx_field = ctx.with_field(&field);
                    let field_name = ctx_field.item.node.response_key().node.clone();

                    let resolve_info = ResolveInfo {
                        resolve_id: ctx_field.resolve_id,
                        path_node: ctx_field.path_node.as_ref().unwrap(),
                        context: &ctx_field,
                        parent_type: &T::type_name(),
                        return_type: match ctx_field
                            .schema_env
                            .registry
                            .types
                            .get(T::type_name().as_ref())
                            .and_then(|ty| ty.field_by_name(&field.node.name.node))
                            .map(|field| &field.ty)
                        {
                            Some(ty) => &ty,
                            None => {
                                return Err(Error::Query {
                                    pos: field.pos,
                                    path: None,
                                    err: QueryError::FieldNotFound {
                                        field_name: field.node.name.node.clone().into_string(),
                                        object: T::type_name().to_string(),
                                    },
                                });
                            }
                        },
                    };

                    ctx_field
                        .query_env
                        .extensions
                        .lock()
                        .resolve_start(&resolve_info);
                    let value = root
                        .resolve_field(&ctx_field)
                        .await
                        .log_error(&ctx.query_env.extensions)?;
                    values.insert(field_name.into_string(), value);

                    ctx_field
                        .query_env
                        .extensions
                        .lock()
                        .resolve_end(&resolve_info);
                }
                Selection::FragmentSpread(fragment_spread) => {
                    if ctx.is_skip(&fragment_spread.node.directives)? {
                        continue;
                    }

                    if let Some(fragment) = ctx
                        .query_env
                        .document
                        .fragments
                        .get(&fragment_spread.node.fragment_name.node)
                    {
                        do_resolve(
                            &ctx.with_selection_set(&fragment.node.selection_set),
                            root,
                            values,
                        )
                        .await?;
                    } else {
                        return Err(Error::Query {
                            pos: fragment_spread.pos,
                            path: None,
                            err: QueryError::UnknownFragment {
                                name: fragment_spread
                                    .node
                                    .fragment_name
                                    .node
                                    .clone()
                                    .into_string(),
                            },
                        });
                    }
                }
                Selection::InlineFragment(inline_fragment) => {
                    if ctx.is_skip(&inline_fragment.node.directives)? {
                        continue;
                    }

                    if let Some(TypeCondition { on: name }) = inline_fragment
                        .node
                        .type_condition
                        .as_ref()
                        .map(|v| &v.node)
                    {
                        let mut futures = Vec::new();
                        root.collect_inline_fields(
                            &name.node,
                            &ctx.with_selection_set(&inline_fragment.node.selection_set),
                            &mut futures,
                        )?;
                        for fut in futures {
                            let (name, value) = fut.await?;
                            values.insert(name, value);
                        }
                    } else {
                        do_resolve(
                            &ctx.with_selection_set(&inline_fragment.node.selection_set),
                            root,
                            values,
                        )
                        .await?;
                    }
                }
            }
        }

        Ok(())
    })
}
