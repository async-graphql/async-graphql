use crate::{ContextSelectionSet, GQLOutputValue, GQLType, Result};
use graphql_parser::query::Selection;
use std::borrow::Cow;

struct QueryRoot<T>(T);

impl<T> GQLType for QueryRoot<T> {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("Root")
    }
}

#[async_trait::async_trait]
impl<T: GQLOutputValue + Send + Sync> GQLOutputValue for QueryRoot<T> {
    async fn resolve(&self, ctx: &ContextSelectionSet<'_>) -> Result<serde_json::Value> {
        let mut value = self.0.resolve(ctx).await?;
        if let serde_json::Value::Object(obj) = &mut value {
            for item in &ctx.item.items {
                if let Selection::Field(field) = item {
                    if field.name == "__schema" {
                        todo!()
                    }
                }
            }
        }
        Ok(value)
    }
}
