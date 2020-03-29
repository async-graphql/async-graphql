use crate::{ContextBase, Result, Type};
use graphql_parser::query::Field;
use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Represents a GraphQL subscription object
#[async_trait::async_trait]
pub trait SubscriptionType: Type {
    /// This function returns true of type `EmptySubscription` only
    #[doc(hidden)]
    fn is_empty() -> bool {
        false
    }

    #[doc(hidden)]
    fn create_type(field: &Field, types: &mut HashMap<TypeId, Field>) -> Result<()>;

    /// Resolve a subscription message, If no message of this type is subscribed, None is returned.
    async fn resolve(
        &self,
        ctx: &ContextBase<'_, ()>,
        types: &HashMap<TypeId, Field>,
        msg: &(dyn Any + Send + Sync),
    ) -> Result<Option<serde_json::Value>>;
}
