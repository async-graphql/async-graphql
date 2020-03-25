use crate::{
    do_resolve, registry, Context, ContextSelectionSet, ErrorWithPosition, ObjectType,
    OutputValueType, Result, Type,
};
use graphql_parser::query::Field;
use std::borrow::Cow;
use std::collections::HashMap;

pub struct Edge<'a, T, E> {
    pub cursor: &'a str,
    pub node: &'a T,
    pub extra_type: &'a E,
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
                registry.types.get_mut(E::type_name().as_ref())
            {
                fields.clone()
            } else {
                unreachable!()
            };
            registry.types.remove(E::type_name().as_ref());

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
                        },
                    );

                    fields.extend(extra_fields);
                    fields
                },
                cache_control: Default::default(),
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
    async fn resolve_field(&self, ctx: &Context<'_>, field: &Field) -> Result<serde_json::Value> {
        if field.name.as_str() == "node" {
            let ctx_obj = ctx.with_item(&field.selection_set);
            return OutputValueType::resolve(self.node, &ctx_obj)
                .await
                .map_err(|err| err.with_position(field.position).into());
        } else if field.name.as_str() == "cursor" {
            return Ok(self.cursor.into());
        }

        self.extra_type.resolve_field(ctx, field).await
    }
}

#[async_trait::async_trait]
impl<'a, T, E> OutputValueType for Edge<'a, T, E>
where
    T: OutputValueType + Send + Sync + 'a,
    E: ObjectType + Sync + Send + 'a,
{
    async fn resolve(value: &Self, ctx: &ContextSelectionSet<'_>) -> Result<serde_json::Value> {
        do_resolve(ctx, value).await
    }
}
