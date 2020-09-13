use crate::parser::types::{Selection, TypeCondition};
use crate::{Context, ContextSelectionSet, Result, Type};
use futures::{Stream, StreamExt};
use std::pin::Pin;

/// Represents a GraphQL subscription object
pub trait SubscriptionType: Type {
    /// This function returns true of type `EmptySubscription` only.
    #[doc(hidden)]
    fn is_empty() -> bool {
        false
    }

    #[doc(hidden)]
    fn create_field_stream<'a>(
        &'a self,
        ctx: &'a Context<'a>,
    ) -> Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send + 'a>>;
}

pub(crate) fn collect_subscription_streams<'a, T: SubscriptionType + Send + Sync + 'static>(
    ctx: &ContextSelectionSet<'a>,
    root: &'a T,
    streams: &mut Vec<Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send + 'a>>>,
) -> Result<()> {
    for selection in &ctx.item.node.items {
        if ctx.is_skip(selection.node.directives())? {
            continue;
        }
        match &selection.node {
            Selection::Field(field) => streams.push(Box::pin({
                let ctx = ctx.clone();
                async_stream::stream! {
                    let ctx = ctx.with_field(field);
                    let mut stream = root.create_field_stream(&ctx);
                    while let Some(item) = stream.next().await {
                        yield item;
                    }
                }
            })),
            Selection::FragmentSpread(fragment_spread) => {
                if let Some(fragment) = ctx
                    .query_env
                    .document
                    .fragments
                    .get(&fragment_spread.node.fragment_name.node)
                {
                    collect_subscription_streams(
                        &ctx.with_selection_set(&fragment.node.selection_set),
                        root,
                        streams,
                    )?;
                }
            }
            Selection::InlineFragment(inline_fragment) => {
                if let Some(TypeCondition { on: name }) = inline_fragment
                    .node
                    .type_condition
                    .as_ref()
                    .map(|v| &v.node)
                {
                    if name.node.as_str() == T::type_name() {
                        collect_subscription_streams(
                            &ctx.with_selection_set(&inline_fragment.node.selection_set),
                            root,
                            streams,
                        )?;
                    }
                } else {
                    collect_subscription_streams(
                        &ctx.with_selection_set(&inline_fragment.node.selection_set),
                        root,
                        streams,
                    )?;
                }
            }
        }
    }
    Ok(())
}

impl<T: SubscriptionType + Send + Sync> SubscriptionType for &T {
    fn create_field_stream<'a>(
        &'a self,
        ctx: &'a Context<'a>,
    ) -> Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send + 'a>> {
        T::create_field_stream(*self, ctx)
    }
}
