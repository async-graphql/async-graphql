use std::borrow::Cow;
use std::pin::Pin;

use futures_util::stream::{Stream, StreamExt};

use crate::parser::types::{Selection, TypeCondition};
use crate::registry::Registry;
use crate::{
    registry, Context, ContextSelectionSet, PathSegment, Response, ServerError, ServerResult,
};

/// A GraphQL subscription object
pub trait SubscriptionType: Send + Sync {
    /// Type the name.
    fn type_name() -> Cow<'static, str>;

    /// Qualified typename.
    fn qualified_type_name() -> String {
        format!("{}!", Self::type_name())
    }

    /// Create type information in the registry and return qualified typename.
    fn create_type_info(registry: &mut registry::Registry) -> String;

    /// This function returns true of type `EmptySubscription` only.
    #[doc(hidden)]
    fn is_empty() -> bool {
        false
    }

    #[doc(hidden)]
    fn create_field_stream<'a>(
        &'a self,
        ctx: &'a Context<'_>,
    ) -> Option<Pin<Box<dyn Stream<Item = Response> + Send + 'a>>>;
}

type BoxFieldStream<'a> = Pin<Box<dyn Stream<Item = Response> + 'a + Send>>;

pub(crate) fn collect_subscription_streams<'a, T: SubscriptionType + 'static>(
    ctx: &ContextSelectionSet<'a>,
    root: &'a T,
    streams: &mut Vec<BoxFieldStream<'a>>,
) -> ServerResult<()> {
    for selection in &ctx.item.node.items {
        if ctx.is_skip(selection.node.directives())? {
            continue;
        }
        match &selection.node {
            Selection::Field(field) => streams.push(Box::pin({
                let ctx = ctx.clone();
                async_stream::stream! {
                    let ctx = ctx.with_field(field);
                    let field_name = ctx.item.node.response_key().node.clone();
                    let stream = root.create_field_stream(&ctx);
                    if let Some(mut stream) = stream {
                        while let Some(resp) = stream.next().await {
                            yield resp;
                        }
                    } else {
                        let err = ServerError::new(format!(r#"Cannot query field "{}" on type "{}"."#, field_name, T::type_name()), Some(ctx.item.pos))
                            .with_path(vec![PathSegment::Field(field_name.to_string())]);
                        yield Response::from_errors(vec![err]);
                    }
                }
            })),
            Selection::FragmentSpread(fragment_spread) => {
                if let Some(fragment) = ctx
                    .query_env
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

impl<T: SubscriptionType> SubscriptionType for &T {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut Registry) -> String {
        T::create_type_info(registry)
    }

    fn create_field_stream<'a>(
        &'a self,
        ctx: &'a Context<'_>,
    ) -> Option<Pin<Box<dyn Stream<Item = Response> + Send + 'a>>> {
        T::create_field_stream(*self, ctx)
    }
}
