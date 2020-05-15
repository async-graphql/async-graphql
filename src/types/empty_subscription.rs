use crate::context::Environment;
use crate::{
    registry, Context, ContextSelectionSet, Error, ObjectType, OutputValueType, Pos, QueryError,
    Result, Schema, SubscriptionType, Type,
};
use futures::Stream;
use std::borrow::Cow;
use std::pin::Pin;
use std::sync::Arc;

/// Empty subscription
///
/// Only the parameters used to construct the Schema, representing an unconfigured subscription.
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

    async fn create_field_stream<Query, Mutation>(
        &self,
        _idx: usize,
        _ctx: &Context<'_>,
        _schema: &Schema<Query, Mutation, Self>,
        _environment: Arc<Environment>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send>>>
    where
        Query: ObjectType + Send + Sync + 'static,
        Mutation: ObjectType + Send + Sync + 'static,
        Self: Send + Sync + 'static + Sized,
    {
        Err(Error::Query {
            pos: Pos::default(),
            path: None,
            err: QueryError::NotConfiguredSubscriptions,
        })
    }
}

#[async_trait::async_trait]
impl OutputValueType for EmptySubscription {
    async fn resolve(&self, _ctx: &ContextSelectionSet<'_>, pos: Pos) -> Result<serde_json::Value> {
        Err(Error::Query {
            pos,
            path: None,
            err: QueryError::NotConfiguredSubscriptions,
        })
    }
}
