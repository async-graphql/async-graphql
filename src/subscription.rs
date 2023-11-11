use std::{borrow::Cow, pin::Pin};

use futures_util::stream::{Stream, StreamExt};

use crate::{
    parser::types::Selection, registry, registry::Registry, Context, ContextSelectionSet,
    PathSegment, Response, ServerError, ServerResult,
};

/// A GraphQL subscription object
pub trait SubscriptionType {
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
    ) -> Option<Pin<Box<dyn Stream<Item = Response> + 'a>>>;
}

pub(crate) type BoxFieldStream<'a> = Pin<Box<dyn Stream<Item = Response> + 'a>>;

pub(crate) fn collect_subscription_streams<'a, T: SubscriptionType + 'static>(
    ctx: &ContextSelectionSet<'a>,
    root: &'a T,
    streams: &mut Vec<BoxFieldStream<'a>>,
) -> ServerResult<()> {
    for selection in &ctx.item.node.items {
        if let Selection::Field(field) = &selection.node {
            streams.push(Box::pin({
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
            }))
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
    ) -> Option<Pin<Box<dyn Stream<Item = Response> + 'a>>> {
        T::create_field_stream(*self, ctx)
    }
}
