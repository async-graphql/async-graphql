use crate::{schema, ContextSelectionSet, GQLOutputValue, GQLType, Result};
use graphql_parser::query::Selection;
use std::borrow::Cow;

struct QueryRoot<T>(T);

impl<T: GQLType> GQLType for QueryRoot<T> {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut schema::Registry) -> String {
        T::create_type_info(registry)
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
