use crate::model::{__Schema, __Type};
use crate::{
    registry, Context, ContextSelectionSet, ErrorWithPosition, GQLObject, GQLOutputValue, GQLType,
    QueryError, Result, Value,
};
use graphql_parser::query::Field;
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
impl<T: GQLObject + Send + Sync> GQLObject for QueryRoot<T> {
    async fn resolve_field(&self, ctx: &Context<'_>, field: &Field) -> Result<serde_json::Value> {
        if field.name.as_str() == "__schema" {
            let ctx_obj = ctx.with_item(&field.selection_set);
            return GQLOutputValue::resolve(
                &__Schema {
                    registry: &ctx.registry,
                    query_type: &self.query_type,
                    mutation_type: self.mutation_type.as_deref(),
                },
                &ctx_obj,
            )
            .await
            .map_err(|err| err.with_position(field.position).into());
        } else if field.name.as_str() == "__type" {
            let type_name: String = ctx.param_value("name", || Value::Null)?;
            let ctx_obj = ctx.with_item(&field.selection_set);
            return GQLOutputValue::resolve(
                &ctx.registry
                    .types
                    .get(&type_name)
                    .map(|ty| __Type::new_simple(ctx.registry, ty)),
                &ctx_obj,
            )
            .await
            .map_err(|err| err.with_position(field.position).into());
        }

        return self.inner.resolve_field(ctx, field).await;
    }

    async fn resolve_inline_fragment(
        &self,
        name: &str,
        _ctx: &ContextSelectionSet<'_>,
        _result: &mut serde_json::Map<String, serde_json::Value>,
    ) -> Result<()> {
        anyhow::bail!(QueryError::UnrecognizedInlineFragment {
            object: T::type_name().to_string(),
            name: name.to_string(),
        });
    }
}
