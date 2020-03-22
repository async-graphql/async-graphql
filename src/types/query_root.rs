use crate::model::{__Schema, __Type};
use crate::{
    do_resolve, registry, Context, ContextSelectionSet, ErrorWithPosition, ObjectType,
    OutputValueType, QueryError, Result, Type, Value,
};
use graphql_parser::query::Field;
use std::borrow::Cow;
use std::collections::HashMap;

pub struct QueryRoot<T> {
    pub inner: T,
}

impl<T: Type> Type for QueryRoot<T> {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        let schema_type = __Schema::create_type_info(registry);
        let root = T::create_type_info(registry);
        if let Some(registry::Type::Object { fields, .. }) =
            registry.types.get_mut(T::type_name().as_ref())
        {
            fields.insert(
                "__schema".to_string(),
                registry::Field {
                    name: "__schema".to_string(),
                    description: Some("Access the current type schema of this server."),
                    args: Default::default(),
                    ty: schema_type,
                    deprecation: None,
                    cache_control: Default::default(),
                },
            );

            fields.insert(
                "__type".to_string(),
                registry::Field {
                    name: "__type".to_string(),
                    description: Some("Request the type information of a single type."),
                    args: {
                        let mut args = HashMap::new();
                        args.insert(
                            "name",
                            registry::InputValue {
                                name: "name",
                                description: None,
                                ty: "String!".to_string(),
                                default_value: None,
                                validator: None,
                            },
                        );
                        args
                    },
                    ty: "__Type".to_string(),
                    deprecation: None,
                    cache_control: Default::default(),
                },
            );
        }
        root
    }
}

#[async_trait::async_trait]
impl<T: ObjectType + Send + Sync> ObjectType for QueryRoot<T> {
    async fn resolve_field(&self, ctx: &Context<'_>, field: &Field) -> Result<serde_json::Value> {
        if field.name.as_str() == "__schema" {
            let ctx_obj = ctx.with_item(&field.selection_set);
            return OutputValueType::resolve(
                &__Schema {
                    registry: &ctx.registry,
                },
                &ctx_obj,
            )
            .await
            .map_err(|err| err.with_position(field.position).into());
        } else if field.name.as_str() == "__type" {
            let type_name: String = ctx.param_value("name", || Value::Null)?;
            let ctx_obj = ctx.with_item(&field.selection_set);
            return OutputValueType::resolve(
                &ctx.registry
                    .types
                    .get(&type_name)
                    .map(|ty| __Type::new_simple(ctx.registry, ty)),
                &ctx_obj,
            )
            .await
            .map_err(|err| err.with_position(field.position).into());
        }

        self.inner.resolve_field(ctx, field).await
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

#[async_trait::async_trait]
impl<T: ObjectType + Send + Sync> OutputValueType for QueryRoot<T> {
    async fn resolve(value: &Self, ctx: &ContextSelectionSet<'_>) -> Result<serde_json::Value> {
        do_resolve(ctx, value).await
    }
}
