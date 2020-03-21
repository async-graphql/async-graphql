use crate::{ContextSelectionSet, ErrorWithPosition, ObjectType, QueryError, Result};
use graphql_parser::query::{Selection, TypeCondition};
use std::future::Future;
use std::pin::Pin;

struct Resolver<'a, T> {
    ctx: &'a ContextSelectionSet<'a>,
    obj: &'a T,
    result: &'a mut serde_json::Map<String, serde_json::Value>,
}

impl<'a, T: ObjectType + Send + Sync> Resolver<'a, T> {
    pub fn resolve(&'a mut self) -> Pin<Box<dyn Future<Output = Result<()>> + 'a + Send>> {
        Box::pin(async move {
            if self.ctx.items.is_empty() {
                anyhow::bail!(QueryError::MustHaveSubFields {
                    object: T::type_name().to_string(),
                }
                .with_position(self.ctx.span.0));
            }

            for selection in &self.ctx.item.items {
                match selection {
                    Selection::Field(field) => {
                        if self.ctx.is_skip(&field.directives)? {
                            continue;
                        }

                        let ctx_field = self.ctx.with_item(field);
                        if field.name.as_str() == "__typename" {
                            self.result
                                .insert(ctx_field.result_name(), T::type_name().to_string().into());
                            continue;
                        }

                        self.result.insert(
                            ctx_field.result_name(),
                            self.obj.resolve_field(&ctx_field, field).await?,
                        );
                    }
                    Selection::FragmentSpread(fragment_spread) => {
                        if self.ctx.is_skip(&fragment_spread.directives)? {
                            continue;
                        }

                        if let Some(fragment) =
                            self.ctx.fragments.get(&fragment_spread.fragment_name)
                        {
                            Resolver {
                                ctx: &self.ctx.with_item(&fragment.selection_set),
                                obj: self.obj,
                                result: self.result,
                            }
                            .resolve()
                            .await?;
                        } else {
                            return Err(QueryError::UnknownFragment {
                                name: fragment_spread.fragment_name.clone(),
                            }
                            .into());
                        }
                    }
                    Selection::InlineFragment(inline_fragment) => {
                        if self.ctx.is_skip(&inline_fragment.directives)? {
                            continue;
                        }

                        if let Some(TypeCondition::On(name)) = &inline_fragment.type_condition {
                            self.obj
                                .resolve_inline_fragment(
                                    &name,
                                    &self.ctx.with_item(&inline_fragment.selection_set),
                                    self.result,
                                )
                                .await?;
                        }
                    }
                }
            }

            Ok(())
        })
    }
}

#[allow(missing_docs)]
pub async fn do_resolve<'a, T: ObjectType + Send + Sync>(
    ctx: &'a ContextSelectionSet<'a>,
    root: &'a T,
) -> Result<serde_json::Value> {
    let mut result = serde_json::Map::<String, serde_json::Value>::new();
    Resolver {
        ctx,
        obj: root,
        result: &mut result,
    }
    .resolve()
    .await?;
    Ok(result.into())
}

#[allow(missing_docs)]
pub async fn do_resolve_values<'a, T: ObjectType + Send + Sync>(
    ctx: &'a ContextSelectionSet<'a>,
    root: &'a T,
    result: &mut serde_json::Map<String, serde_json::Value>,
) -> Result<()> {
    Resolver {
        ctx,
        obj: root,
        result,
    }
    .resolve()
    .await?;
    Ok(())
}
