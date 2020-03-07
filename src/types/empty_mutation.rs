use crate::{registry, Context, ContextSelectionSet, GQLObject, GQLType, QueryError, Result};
use graphql_parser::query::Field;
use serde_json::{Map, Value};
use std::borrow::Cow;

pub struct GQLEmptyMutation;

impl GQLType for GQLEmptyMutation {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("EmptyMutation")
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        registry.create_type::<Self, _>(|_| registry::Type::Object {
            name: "EmptyMutation",
            description: None,
            fields: Vec::new(),
        })
    }
}

#[async_trait::async_trait]
impl GQLObject for GQLEmptyMutation {
    fn is_empty() -> bool {
        return true;
    }

    async fn resolve_field(&self, _ctx: &Context<'_>, _name: &Field) -> Result<serde_json::Value> {
        return Err(QueryError::NotConfiguredMutations.into());
    }

    async fn resolve_inline_fragment(
        &self,
        _name: &str,
        _ctx: &ContextSelectionSet<'_>,
        _result: &mut Map<String, Value>,
    ) -> Result<()> {
        return Err(QueryError::NotConfiguredMutations.into());
    }
}
