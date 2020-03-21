use crate::{
    registry, ContextBase, ContextSelectionSet, OutputValueType, QueryError, Result,
    SubscriptionType, Type,
};
use graphql_parser::query::Field;
use serde_json::Value;
use std::any::{Any, TypeId};
use std::borrow::Cow;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;

/// Empty subscription
///
/// Only the parameters used to construct the Schema, representing an unconfigured subscription.
pub struct EmptySubscription;

impl Type for EmptySubscription {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("EmptyMutation")
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        registry.create_type::<Self, _>(|_| registry::Type::Object {
            name: "EmptySubscription".to_string(),
            description: None,
            fields: Default::default(),
        })
    }
}

#[async_trait::async_trait]
impl SubscriptionType for EmptySubscription {
    fn is_empty() -> bool {
        true
    }

    fn create_type(_field: &Field, _types: &mut HashMap<TypeId, Field>) -> Result<()> {
        unreachable!()
    }

    async fn resolve(
        &self,
        _ctx: &ContextBase<'_, ()>,
        _types: &HashMap<TypeId, Field, RandomState>,
        _msg: &(dyn Any + Send + Sync),
    ) -> Result<Option<Value>> {
        unreachable!()
    }
}

#[async_trait::async_trait]
impl OutputValueType for EmptySubscription {
    async fn resolve(_value: &Self, _ctx: &ContextSelectionSet<'_>) -> Result<serde_json::Value> {
        Err(QueryError::NotConfiguredSubscriptions.into())
    }
}
