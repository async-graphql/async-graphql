use crate::context::QueryEnv;
use crate::{registry, Context, Error, Pos, QueryError, Result, SchemaEnv, SubscriptionType, Type};
use futures::Stream;
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

#[async_trait::async_trait]
impl SubscriptionType for EmptySubscription {
    fn is_empty() -> bool {
        true
    }

    async fn create_field_stream(
        &self,
        _idx: usize,
        _ctx: &Context<'_>,
        _schema_env: SchemaEnv,
        _query_env: QueryEnv,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send>>>
    where
        Self: Send + Sync + 'static + Sized,
    {
        Err(Error::Query {
            pos: Pos::default(),
            path: None,
            err: QueryError::NotConfiguredSubscriptions,
        })
    }
}
