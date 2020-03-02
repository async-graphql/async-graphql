use crate::{
    ContextSelectionSet, ErrorWithPosition, GQLObject, GQLOutputValue, GQLType, QueryError, Result,
};
use std::borrow::Cow;

pub struct GQLEmptyMutation;

impl GQLType for GQLEmptyMutation {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("EmptyMutation")
    }
}

#[async_trait::async_trait]
impl GQLOutputValue for GQLEmptyMutation {
    async fn resolve(&self, ctx: &ContextSelectionSet<'_>) -> Result<serde_json::Value> {
        anyhow::bail!(QueryError::NotConfiguredMutations.with_position(ctx.item.span.0));
    }
}

impl GQLObject for GQLEmptyMutation {}
