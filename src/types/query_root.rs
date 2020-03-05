use crate::model::__Schema;
use crate::{registry, ContextSelectionSet, GQLOutputValue, GQLType, Result};
use graphql_parser::query::Selection;
use std::borrow::Cow;

pub struct QueryRoot<T> {
    pub inner: T,
    pub query_type: String,
    pub mutation_type: Option<String>,
}

impl<T: GQLType> GQLType for QueryRoot<T> {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        T::create_type_info(registry)
    }
}

#[async_trait::async_trait]
impl<T: GQLOutputValue + Send + Sync> GQLOutputValue for QueryRoot<T> {
    async fn resolve(&self, ctx: &ContextSelectionSet<'_>) -> Result<serde_json::Value> {
        let mut res = self.inner.resolve(ctx).await?;

        if let serde_json::Value::Object(obj) = &mut res {
            for item in &ctx.item.items {
                if let Selection::Field(field) = item {
                    if field.name == "__schema" {
                        let ctx_obj = ctx.with_item(&field.selection_set);
                        obj.insert(
                            "__schema".to_string(),
                            __Schema {
                                registry: &ctx.registry,
                                query_type: &self.query_type,
                                mutation_type: self.mutation_type.as_deref(),
                            }
                            .resolve(&ctx_obj)
                            .await?,
                        );
                    }
                }
            }
        }

        Ok(res)
    }
}
