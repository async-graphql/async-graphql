use crate::{ContextSelectionSet, ErrorWithPosition, GQLObject, QueryError, Result};
use graphql_parser::query::Selection;
use std::future::Future;
use std::pin::Pin;

struct Resolver<'a, T> {
    ctx: &'a ContextSelectionSet<'a>,
    obj: &'a T,
    result: &'a mut serde_json::Map<String, serde_json::Value>,
}

impl<'a, T: GQLObject + Send + Sync> Resolver<'a, T> {
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
                        let ctx_field = self.ctx.with_item(field);
                        if ctx_field.is_skip(&field.directives)? {
                            continue;
                        }

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
                    Selection::InlineFragment(_) => {}
                }
            }

            Ok(())
        })
    }
}

pub async fn do_resolve<'a, T: GQLObject + Send + Sync>(
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

    Ok(serde_json::Value::Object(result))
}
