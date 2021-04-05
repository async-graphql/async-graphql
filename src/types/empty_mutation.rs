use crate::parser::types::Field;
use crate::resolver_utils::ContainerType;
use crate::{
    registry, Context, ContextSelectionSet, ObjectType, OutputType, Positioned, ServerError,
    ServerResult, Type, Value,
};

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
/// impl QueryRoot {
///     async fn value(&self) -> i32 {
///         // A GraphQL Object type must define one or more fields.
///         100
///     }
/// }
///
/// let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
/// ```
#[derive(Default, Copy, Clone)]
pub struct EmptyMutation;

impl Type for EmptyMutation {
    fn type_name() -> &'static str {
        "EmptyMutation"
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        registry.create_type::<Self, _>(|_| registry::MetaType::Object {
            name: "EmptyMutation".to_string(),
            description: None,
            fields: Default::default(),
            cache_control: Default::default(),
            extends: false,
            keys: None,
            visible: None,
        })
    }
}

#[async_trait::async_trait]
impl ContainerType for EmptyMutation {
    fn is_empty() -> bool {
        true
    }

    async fn resolve_field(&self, _ctx: &Context<'_>) -> ServerResult<Option<Value>> {
        unreachable!()
    }
}

#[async_trait::async_trait]
impl OutputType for EmptyMutation {
    async fn resolve(
        &self,
        _ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        Err(ServerError::new("Schema is not configured for mutations.").at(field.pos))
    }
}

impl ObjectType for EmptyMutation {}
