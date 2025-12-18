use std::borrow::Cow;

use crate::{
    Any, Context, ContextSelectionSet, ObjectType, OutputType, OutputTypeMarker, Positioned,
    ServerError, ServerResult, SimpleObject, Value,
    model::{__Schema, __Type},
    parser::types::Field,
    registry::{self, SDLExportOptions},
    resolver_utils::{ContainerType, resolve_container},
    schema::IntrospectionMode,
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

#[cfg_attr(feature = "boxed-trait", async_trait::async_trait)]
impl<T: ObjectType> ContainerType for QueryRoot<T> {
    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<Value>> {
        if matches!(
            ctx.schema_env.registry.introspection_mode,
            IntrospectionMode::Enabled | IntrospectionMode::IntrospectionOnly
        ) && matches!(
            ctx.query_env.introspection_mode,
            IntrospectionMode::Enabled | IntrospectionMode::IntrospectionOnly,
        ) {
            if ctx.item.node.name.node == "__schema" {
                let mut ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                ctx_obj.is_for_introspection = true;
                let visible_types = ctx.schema_env.registry.find_visible_types(ctx);
                return OutputType::resolve(
                    &__Schema::new(&ctx.schema_env.registry, &visible_types),
                    &ctx_obj,
                    ctx.item,
                )
                .await
                .map(Some);
            } else if ctx.item.node.name.node == "__type" {
                let (_, type_name) = ctx.param_value::<String>("name", None)?;
                let mut ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                ctx_obj.is_for_introspection = true;
                let visible_types = ctx.schema_env.registry.find_visible_types(ctx);
                return OutputType::resolve(
                    &ctx.schema_env
                        .registry
                        .types
                        .get(&type_name)
                        .filter(|_| visible_types.contains(type_name.as_str()))
                        .map(|ty| __Type::new_simple(&ctx.schema_env.registry, &visible_types, ty)),
                    &ctx_obj,
                    ctx.item,
                )
                .await
                .map(Some);
            }
        }

        if ctx.schema_env.registry.introspection_mode == IntrospectionMode::IntrospectionOnly
            || ctx.query_env.introspection_mode == IntrospectionMode::IntrospectionOnly
        {
            return Ok(None);
        }

        if ctx.schema_env.registry.enable_federation || ctx.schema_env.registry.has_entities() {
            if ctx.item.node.name.node == "_entities" {
                let (_, representations) = ctx.param_value::<Vec<Any>>("representations", None)?;
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
                let mut ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                ctx_obj.is_for_introspection = true;
                return OutputType::resolve(
                    &Service {
                        sdl: Some(
                            ctx.schema_env.registry.export_sdl(
                                SDLExportOptions::new().federation().compose_directive(),
                            ),
                        ),
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

impl<T: OutputTypeMarker> OutputTypeMarker for QueryRoot<T> {
    fn type_name() -> Cow<'static, str> {
        <T as OutputTypeMarker>::type_name()
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        let root = <T as OutputTypeMarker>::create_type_info(registry);

        if matches!(
            registry.introspection_mode,
            IntrospectionMode::Enabled | IntrospectionMode::IntrospectionOnly
        ) {
            registry.create_introspection_types();
        }

        root
    }
}

#[cfg_attr(feature = "boxed-trait", async_trait::async_trait)]
impl<T: ObjectType> OutputType for QueryRoot<T> {
    #[cfg(feature = "boxed-trait")]
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        _field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        resolve_container(ctx, self, self).await
    }

    #[cfg(not(feature = "boxed-trait"))]
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        _field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        resolve_container(ctx, self).await
    }
}

impl<T: ObjectType> ObjectType for QueryRoot<T> {}
