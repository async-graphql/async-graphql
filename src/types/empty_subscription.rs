use crate::type_mark::TypeMarkSubscription;
use crate::{registry, Context, ServerError, ServerResult, SubscriptionType, Type};
use futures::{stream, Stream};
use std::borrow::Cow;
use std::pin::Pin;

/// Empty subscription
///
/// Only the parameters used to construct the Schema, representing an unconfigured subscription.
#[derive(Default, Copy, Clone)]
pub struct EmptySubscription;

impl Type for EmptySubscription {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("EmptyMutation")
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        registry.create_type::<Self, _>(|_| registry::MetaType::Object {
            name: "EmptySubscription".to_string(),
            description: None,
            fields: Default::default(),
            cache_control: Default::default(),
            extends: false,
            keys: None,
        })
    }
}

impl SubscriptionType for EmptySubscription {
    fn is_empty() -> bool {
        true
    }

    fn create_field_stream<'a>(
        &'a self,
        ctx: &'a Context<'a>,
    ) -> Option<Pin<Box<dyn Stream<Item = ServerResult<serde_json::Value>> + Send + 'a>>>
    where
        Self: Send + Sync + 'static + Sized,
    {
        Some(Box::pin(stream::once(async move {
            Err(ServerError::new("Schema is not configured for mutations.").at(ctx.item.pos))
        })))
    }
}

impl TypeMarkSubscription for EmptySubscription {}
