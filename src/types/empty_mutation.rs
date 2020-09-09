use crate::parser::types::Field;
use crate::{
    registry, Context, ContextSelectionSet, Error, ObjectType, OutputValueType, Positioned,
    QueryError, Result, Type,
};
use std::borrow::Cow;

/// Empty mutation
///
/// Only the parameters used to construct the Schema, representing an unconfigured mutation.
///
/// # Examples
///
/// ```rust
/// use async_graphql::*;
///
/// struct QueryRoot;
///
/// #[Object]
/// impl QueryRoot {}
///
/// let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
/// ```
#[derive(Default, Copy, Clone)]
pub struct EmptyMutation;

impl Type for EmptyMutation {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("EmptyMutation")
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        registry.create_type::<Self, _>(|_| registry::MetaType::Object {
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

    async fn resolve_field(&self, _ctx: &Context<'_>) -> Result<serde_json::Value> {
        unreachable!()
    }
}

#[async_trait::async_trait]
impl OutputValueType for EmptyMutation {
    async fn resolve(
        &self,
        _ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> Result<serde_json::Value> {
        Err(Error::Query {
            pos: field.pos,
            path: None,
            err: QueryError::NotConfiguredMutations,
        })
    }
}
