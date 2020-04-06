use crate::context::Environment;
use crate::{Context, ContextSelectionSet, ObjectType, Result, Schema, Type};
use futures::Stream;
use graphql_parser::query::{Selection, TypeCondition};
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
    fn create_field_stream<Query, Mutation>(
        &self,
        ctx: &Context<'_>,
        schema: &Schema<Query, Mutation, Self>,
        environment: Arc<Environment>,
    ) -> Result<Pin<Box<dyn Stream<Item = serde_json::Value>>>>
    where
        Query: ObjectType + Send + Sync + 'static,
        Mutation: ObjectType + Send + Sync + 'static,
        Self: Send + Sync + 'static + Sized;
}

pub fn create_subscription_stream<Query, Mutation, Subscription>(
    schema: &Schema<Query, Mutation, Subscription>,
    environment: Arc<Environment>,
    ctx: &ContextSelectionSet<'_>,
    streams: &mut Vec<Pin<Box<dyn Stream<Item = serde_json::Value>>>>,
) -> Result<()>
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static + Sized,
{
    for selection in &ctx.items {
        match selection {
            Selection::Field(field) => {
                if ctx.is_skip(&field.directives)? {
                    continue;
                }
                streams.push(schema.0.subscription.create_field_stream(
                    &ctx.with_field(field),
                    schema,
                    environment.clone(),
                )?)
            }
            Selection::FragmentSpread(fragment_spread) => {
                if ctx.is_skip(&fragment_spread.directives)? {
                    continue;
                }

                if let Some(fragment) = ctx.fragments.get(fragment_spread.fragment_name.as_str()) {
                    create_subscription_stream(
                        schema,
                        environment.clone(),
                        &ctx.with_selection_set(&fragment.selection_set),
                        streams,
                    )?;
                }
            }
            Selection::InlineFragment(inline_fragment) => {
                if ctx.is_skip(&inline_fragment.directives)? {
                    continue;
                }

                if let Some(TypeCondition::On(name)) = &inline_fragment.type_condition {
                    if name.as_str() == Subscription::type_name() {
                        create_subscription_stream(
                            schema,
                            environment.clone(),
                            &ctx.with_selection_set(&inline_fragment.selection_set),
                            streams,
                        )?;
                    }
                } else {
                    create_subscription_stream(
                        schema,
                        environment.clone(),
                        &ctx.with_selection_set(&inline_fragment.selection_set),
                        streams,
                    )?;
                }
            }
        }
    }
    Ok(())
}
