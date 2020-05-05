use crate::{
    do_resolve, registry, Context, ContextSelectionSet, ObjectType, OutputValueType, Result, Type,
};
use graphql_parser::Pos;
use std::borrow::Cow;
use std::collections::HashMap;

pub struct Edge<'a, T, E> {
    pub cursor: &'a str,
    pub node: &'a T,
    pub extra_type: &'a E,
}

impl<'a, T, E> Edge<'a, T, E>
where
    T: OutputValueType + Send + Sync + 'a,
    E: ObjectType + Sync + Send + 'a,
{
    #[doc(hidden)]
    #[inline]
    pub async fn node(&self) -> &T {
        self.node
    }

    #[doc(hidden)]
    #[inline]
    pub async fn cursor(&self) -> &str {
        self.cursor
    }
}

impl<'a, T, E> Type for Edge<'a, T, E>
where
    T: OutputValueType + Send + Sync + 'a,
    E: ObjectType + Sync + Send + 'a,
{
    fn type_name() -> Cow<'static, str> {
        Cow::Owned(format!("{}Edge", T::type_name()))
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        registry.create_type::<Self, _>(|registry| {
            E::create_type_info(registry);
            let extra_fields = if let Some(registry::Type::Object { fields, .. }) =
                registry.types.remove(E::type_name().as_ref())
            {
                fields
            } else {
                unreachable!()
            };

            registry::Type::Object {
                name: Self::type_name().to_string(),
                description: Some("An edge in a connection."),
                fields: {
                    let mut fields = HashMap::new();

                    fields.insert(
                        "node".to_string(),
                        registry::Field {
                            name: "node".to_string(),
                            description: Some("The item at the end of the edge"),
                            args: Default::default(),
                            ty: T::create_type_info(registry),
                            deprecation: None,
                            cache_control: Default::default(),
                            external: false,
                            requires: None,
                            provides: None,
                        },
                    );

                    fields.insert(
                        "cursor".to_string(),
                        registry::Field {
                            name: "cursor".to_string(),
                            description: Some("A cursor for use in pagination"),
                            args: Default::default(),
                            ty: String::create_type_info(registry),
                            deprecation: None,
                            cache_control: Default::default(),
                            external: false,
                            requires: None,
                            provides: None,
                        },
                    );

                    fields.extend(extra_fields);
                    fields
                },
                cache_control: Default::default(),
                extends: false,
                keys: None,
            }
        })
    }
}

#[async_trait::async_trait]
impl<'a, T, E> ObjectType for Edge<'a, T, E>
where
    T: OutputValueType + Send + Sync + 'a,
    E: ObjectType + Sync + Send + 'a,
{
    async fn resolve_field(&self, ctx: &Context<'_>) -> Result<serde_json::Value> {
        if ctx.name.as_str() == "node" {
            let ctx_obj = ctx.with_selection_set(&ctx.selection_set);
            return OutputValueType::resolve(self.node().await, &ctx_obj, ctx.position).await;
        } else if ctx.name.as_str() == "cursor" {
            return Ok(self.cursor().await.into());
        }

        self.extra_type.resolve_field(ctx).await
    }
}

#[async_trait::async_trait]
impl<'a, T, E> OutputValueType for Edge<'a, T, E>
where
    T: OutputValueType + Send + Sync + 'a,
    E: ObjectType + Sync + Send + 'a,
{
    async fn resolve(
        value: &Self,
        ctx: &ContextSelectionSet<'_>,
        _pos: Pos,
    ) -> Result<serde_json::Value> {
        do_resolve(ctx, value).await
    }
}
