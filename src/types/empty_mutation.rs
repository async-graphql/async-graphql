use crate::{
    registry, GqlContext, GqlContextSelectionSet, GqlError, GqlResult, ObjectType, OutputValueType,
    Pos, QueryError, Type,
};
use std::borrow::Cow;

/// Empty mutation
///
/// Only the parameters used to construct the Schema, representing an unconfigured mutation.
///
/// # Examples
///
/// ```rust
/// use async_graphql::prelude::*;
/// use async_graphql::{EmptyMutation, EmptySubscription};
///
/// struct QueryRoot;
///
/// #[GqlObject]
/// impl QueryRoot {}
///
/// fn main() {
///     let schema = GqlSchema::new(QueryRoot, EmptyMutation, EmptySubscription);
/// }
/// ```
pub struct EmptyMutation;

impl Type for EmptyMutation {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("EmptyMutation")
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        registry.create_type::<Self, _>(|_| registry::Type::Object {
            name: "EmptyMutation".to_string(),
            description: None,
            fields: Default::default(),
            cache_control: Default::default(),
            extends: false,
            keys: None,
        })
    }
}

#[async_trait::async_trait]
impl ObjectType for EmptyMutation {
    fn is_empty() -> bool {
        true
    }

    async fn resolve_field(&self, _ctx: &GqlContext<'_>) -> GqlResult<serde_json::Value> {
        unreachable!()
    }
}

#[async_trait::async_trait]
impl OutputValueType for EmptyMutation {
    async fn resolve(
        &self,
        _ctx: &GqlContextSelectionSet<'_>,
        pos: Pos,
    ) -> GqlResult<serde_json::Value> {
        Err(GqlError::Query {
            pos,
            path: None,
            err: QueryError::NotConfiguredMutations,
        })
    }
}
