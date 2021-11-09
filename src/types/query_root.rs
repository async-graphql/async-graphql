use std::borrow::Cow;

use indexmap::map::IndexMap;

use crate::model::{__Schema, __Type};
use crate::parser::types::Field;
use crate::resolver_utils::{resolve_container, ContainerType};
use crate::{
    registry, Any, Context, ContextSelectionSet, ObjectType, OutputType, Positioned, ServerError,
    ServerResult, SimpleObject, Value,
};

/// Federation service
#[derive(SimpleObject)]
#[graphql(internal, name = "_Service")]
struct Service {
    sdl: Option<String>,
}

pub(crate) struct QueryRoot<T> {
    pub(crate) inner: T,
}

#[async_trait::async_trait]
impl<T: ObjectType> ContainerType for QueryRoot<T> {
    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<Value>> {
        if !ctx.schema_env.registry.disable_introspection && !ctx.query_env.disable_introspection {
            if ctx.item.node.name.node == "__schema" {
                let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                return OutputType::resolve(
                    &__Schema {
                        registry: &ctx.schema_env.registry,
                    },
                    &ctx_obj,
                    ctx.item,
                )
                .await
                .map(Some);
            } else if ctx.item.node.name.node == "__type" {
                let type_name: String = ctx.param_value("name", None)?;
                let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                return OutputType::resolve(
                    &ctx.schema_env
                        .registry
                        .types
                        .get(&type_name)
                        .filter(|ty| ty.is_visible(ctx))
                        .map(|ty| __Type::new_simple(&ctx.schema_env.registry, ty)),
                    &ctx_obj,
                    ctx.item,
                )
                .await
                .map(Some);
            }
        }

        if ctx.schema_env.registry.enable_federation || ctx.schema_env.registry.has_entities() {
            if ctx.item.node.name.node == "_entities" {
                let representations: Vec<Any> = ctx.param_value("representations", None)?;
                let res = futures_util::future::try_join_all(representations.iter().map(
                    |item| async move {
                        self.inner.find_entity(ctx, &item.0).await?.ok_or_else(|| {
                            ServerError::new("Entity not found.", Some(ctx.item.pos))
                        })
                    },
                ))
                .await?;
                return Ok(Some(Value::List(res)));
            } else if ctx.item.node.name.node == "_service" {
                let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                return OutputType::resolve(
                    &Service {
                        sdl: Some(ctx.schema_env.registry.export_sdl(true)),
                    },
                    &ctx_obj,
                    ctx.item,
                )
                .await
                .map(Some);
            }
        }

        self.inner.resolve_field(ctx).await
    }
}

#[async_trait::async_trait]
impl<T: ObjectType> OutputType for QueryRoot<T> {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        let root = T::create_type_info(registry);

        if !registry.disable_introspection {
            let schema_type = __Schema::create_type_info(registry);
            if let Some(registry::MetaType::Object { fields, .. }) =
                registry.types.get_mut(T::type_name().as_ref())
            {
                fields.insert(
                    "__schema".to_string(),
                    registry::MetaField {
                        name: "__schema".to_string(),
                        description: Some("Access the current type schema of this server."),
                        args: Default::default(),
                        ty: schema_type,
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                        visible: None,
                        compute_complexity: None,
                    },
                );

                fields.insert(
                    "__type".to_string(),
                    registry::MetaField {
                        name: "__type".to_string(),
                        description: Some("Request the type information of a single type."),
                        args: {
                            let mut args = IndexMap::new();
                            args.insert(
                                "name",
                                registry::MetaInputValue {
                                    name: "name",
                                    description: None,
                                    ty: "String!".to_string(),
                                    default_value: None,
                                    validator: None,
                                    visible: None,
                                    is_secret: false,
                                },
                            );
                            args
                        },
                        ty: "__Type".to_string(),
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                        visible: None,
                        compute_complexity: None,
                    },
                );
            }
        }

        root
    }

    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        _field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        resolve_container(ctx, self).await
    }
}

impl<T: ObjectType> ObjectType for QueryRoot<T> {}
