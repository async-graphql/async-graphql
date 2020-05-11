use crate::context::Environment;
use crate::parser::ast::{Selection, TypeCondition};
use crate::{GqlContext, GqlContextSelectionSet, GqlResult, GqlSchema, ObjectType, Type};
use futures::{Future, Stream};
use std::pin::Pin;
use std::sync::Arc;

/// Represents a GraphQL subscription object
#[async_trait::async_trait]
pub trait SubscriptionType: Type {
    /// This function returns true of type `EmptySubscription` only
    #[doc(hidden)]
    fn is_empty() -> bool {
        false
    }

    #[doc(hidden)]
    async fn create_field_stream<Query, Mutation>(
        &self,
        ctx: &GqlContext<'_>,
        schema: &GqlSchema<Query, Mutation, Self>,
        environment: Arc<Environment>,
    ) -> GqlResult<Pin<Box<dyn Stream<Item = GqlResult<serde_json::Value>> + Send>>>
    where
        Query: ObjectType + Send + Sync + 'static,
        Mutation: ObjectType + Send + Sync + 'static,
        Self: Send + Sync + 'static + Sized;
}

type BoxCreateStreamFuture<'a> = Pin<Box<dyn Future<Output = GqlResult<()>> + Send + 'a>>;

pub fn create_subscription_stream<'a, Query, Mutation, Subscription>(
    schema: &'a GqlSchema<Query, Mutation, Subscription>,
    environment: Arc<Environment>,
    ctx: &'a GqlContextSelectionSet<'_>,
    streams: &'a mut Vec<Pin<Box<dyn Stream<Item = GqlResult<serde_json::Value>> + Send>>>,
) -> BoxCreateStreamFuture<'a>
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static + Sized,
{
    Box::pin(async move {
        for selection in &ctx.items {
            match &selection.node {
                Selection::Field(field) => {
                    if ctx.is_skip(&field.directives)? {
                        continue;
                    }
                    streams.push(
                        schema
                            .0
                            .subscription
                            .create_field_stream(
                                &ctx.with_field(field),
                                schema,
                                environment.clone(),
                            )
                            .await?,
                    )
                }
                Selection::FragmentSpread(fragment_spread) => {
                    if ctx.is_skip(&fragment_spread.directives)? {
                        continue;
                    }

                    if let Some(fragment) =
                        ctx.fragments.get(fragment_spread.fragment_name.as_str())
                    {
                        create_subscription_stream(
                            schema,
                            environment.clone(),
                            &ctx.with_selection_set(&fragment.selection_set),
                            streams,
                        )
                        .await?;
                    }
                }
                Selection::InlineFragment(inline_fragment) => {
                    if ctx.is_skip(&inline_fragment.directives)? {
                        continue;
                    }

                    if let Some(TypeCondition::On(name)) =
                        inline_fragment.type_condition.as_ref().map(|v| &v.node)
                    {
                        if name.as_str() == Subscription::type_name() {
                            create_subscription_stream(
                                schema,
                                environment.clone(),
                                &ctx.with_selection_set(&inline_fragment.selection_set),
                                streams,
                            )
                            .await?;
                        }
                    } else {
                        create_subscription_stream(
                            schema,
                            environment.clone(),
                            &ctx.with_selection_set(&inline_fragment.selection_set),
                            streams,
                        )
                        .await?;
                    }
                }
            }
        }
        Ok(())
    })
}
