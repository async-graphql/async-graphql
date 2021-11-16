use std::borrow::Cow;
use std::pin::Pin;

use futures_util::stream::{self, Stream};

use crate::{registry, Context, Response, ServerError, SubscriptionType};

/// Empty subscription
///
/// Only the parameters used to construct the Schema, representing an unconfigured subscription.
#[derive(Default, Copy, Clone)]
pub struct EmptySubscription;

impl SubscriptionType for EmptySubscription {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("EmptyMutation")
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        registry.create_subscription_type::<Self, _>(|_| registry::MetaType::Object {
            name: "EmptySubscription".to_string(),
            description: None,
            fields: Default::default(),
            cache_control: Default::default(),
            extends: false,
            keys: None,
            visible: None,
            is_subscription: true,
            rust_typename: std::any::type_name::<Self>(),
        })
    }

    fn is_empty() -> bool {
        true
    }

    fn create_field_stream<'a>(
        &'a self,
        _ctx: &'a Context<'_>,
    ) -> Option<Pin<Box<dyn Stream<Item = Response> + Send + 'a>>>
    where
        Self: Send + Sync + 'static + Sized,
    {
        Some(Box::pin(stream::once(async move {
            let err = ServerError::new("Schema is not configured for subscription.", None);
            Response::from_errors(vec![err])
        })))
    }
}
