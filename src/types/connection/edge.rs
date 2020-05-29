use crate::connection::EmptyFields;
use crate::types::connection::CursorType;
use crate::{
    do_resolve, registry, Context, ContextSelectionSet, ObjectType, OutputValueType, Positioned,
    QueryError, Result, Type,
};
use async_graphql_parser::query::Field;
use indexmap::map::IndexMap;
use std::borrow::Cow;

/// The edge type output by the data source
pub struct Edge<C, T, E> {
    pub(crate) cursor: C,
    pub(crate) node: T,
    pub(crate) additional_fields: E,
}

impl<C, T, E> Edge<C, T, E> {
    /// Create a new edge, it can have some additional fields.
    pub fn with_additional_fields(cursor: C, node: T, additional_fields: E) -> Self {
        Self {
            cursor,
            additional_fields,
            node,
        }
    }
}

impl<C: CursorType, T> Edge<C, T, EmptyFields> {
    /// Create a new edge.
    pub fn new(cursor: C, node: T) -> Self {
        Self {
            cursor,
            node,
            additional_fields: EmptyFields,
        }
    }
}

impl<C, T, E> Type for Edge<C, T, E>
where
    C: CursorType,
    T: OutputValueType + Send + Sync,
    E: ObjectType + Sync + Send,
{
    fn type_name() -> Cow<'static, str> {
        Cow::Owned(format!("{}Edge", T::type_name()))
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        registry.create_type::<Self, _>(|registry| {
            E::create_type_info(registry);
            let additional_fields = if let Some(registry::MetaType::Object { fields, .. }) =
                registry.types.remove(E::type_name().as_ref())
            {
                fields
            } else {
                unreachable!()
            };

            registry::MetaType::Object {
                name: Self::type_name().to_string(),
                description: Some("An edge in a connection."),
                fields: {
                    let mut fields = IndexMap::new();

                    fields.insert(
                        "node".to_string(),
                        registry::MetaField {
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
                        registry::MetaField {
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

                    fields.extend(additional_fields);
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
impl<C, T, E> ObjectType for Edge<C, T, E>
where
    C: CursorType + Send + Sync,
    T: OutputValueType + Send + Sync,
    E: ObjectType + Sync + Send,
{
    async fn resolve_field(&self, ctx: &Context<'_>) -> Result<serde_json::Value> {
        if ctx.name.node == "node" {
            let ctx_obj = ctx.with_selection_set(&ctx.selection_set);
            return OutputValueType::resolve(&self.node, &ctx_obj, ctx.item).await;
        } else if ctx.name.node == "cursor" {
            return Ok(self
                .cursor
                .encode_cursor()
                .map_err(|err| {
                    QueryError::FieldError {
                        err: err.to_string(),
                        extended_error: None,
                    }
                    .into_error(ctx.position())
                })?
                .into());
        }

        self.additional_fields.resolve_field(ctx).await
    }
}

#[async_trait::async_trait]
impl<C, T, E> OutputValueType for Edge<C, T, E>
where
    C: CursorType + Send + Sync,
    T: OutputValueType + Send + Sync,
    E: ObjectType + Sync + Send,
{
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        _field: &Positioned<Field>,
    ) -> Result<serde_json::Value> {
        do_resolve(ctx, self).await
    }
}
