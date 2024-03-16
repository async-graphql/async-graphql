use std::{borrow::Cow, pin::Pin};

use futures_util::stream::{self, Stream};

use crate::{registry, Context, Response, ServerError, SubscriptionType};

/// Empty subscription
///
/// Only the parameters used to construct the Schema, representing an
/// unconfigured subscription.
#[derive(Default, Copy, Clone)]
pub struct EmptySubscription;

impl SubscriptionType for EmptySubscription {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("EmptySubscription")
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        registry.create_subscription_type::<Self, _>(|_| registry::MetaType::Object {
            name: "EmptySubscription".to_string(),
            description: None,
            fields: Default::default(),
            cache_control: Default::default(),
            extends: false,
            shareable: false,
            resolvable: true,
            keys: None,
            visible: None,
            inaccessible: false,
            interface_object: false,
            tags: Default::default(),
            is_subscription: true,
            rust_typename: Some(std::any::type_name::<Self>()),
            directive_invocations: Default::default(),
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
