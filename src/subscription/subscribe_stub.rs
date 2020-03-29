use crate::{ContextBase, ObjectType, Result, Schema, SubscriptionType, Variables};
use graphql_parser::query::{Field, FragmentDefinition, VariableDefinition};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;

/// Subscription stub
///
/// When a new push message is generated, a JSON object that needs to be pushed can be obtained by
/// `Subscribe::resolve`, and if None is returned, the Subscribe is not subscribed to a message of this type.
pub struct SubscriptionStub<Query, Mutation, Subscription> {
    pub(crate) schema: Schema<Query, Mutation, Subscription>,
    pub(crate) types: HashMap<TypeId, Field>,
    pub(crate) variables: Variables,
    pub(crate) variable_definitions: Vec<VariableDefinition>,
    pub(crate) fragments: HashMap<String, FragmentDefinition>,
}

impl<Query, Mutation, Subscription> SubscriptionStub<Query, Mutation, Subscription>
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    #[doc(hidden)]
    pub async fn resolve(
        &self,
        msg: &(dyn Any + Send + Sync),
    ) -> Result<Option<serde_json::Value>> {
        let resolve_id = AtomicUsize::default();
        let ctx = ContextBase::<()> {
            path_node: None,
            extensions: &[],
            item: (),
            resolve_id: &resolve_id,
            variables: &self.variables,
            variable_definitions: Some(&self.variable_definitions),
            registry: &self.schema.0.registry,
            data: &self.schema.0.data,
            fragments: &self.fragments,
        };
        self.schema
            .0
            .subscription
            .resolve(&ctx, &self.types, msg)
            .await
    }
}
