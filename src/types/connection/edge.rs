use std::borrow::Cow;

use indexmap::map::IndexMap;

use crate::connection::EmptyFields;
use crate::parser::types::Field;
use crate::resolver_utils::{resolve_container, ContainerType};
use crate::types::connection::CursorType;
use crate::{
    registry, Context, ContextSelectionSet, ObjectType, OutputType, Positioned, ServerResult, Value,
};

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
            node,
            additional_fields,
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

#[async_trait::async_trait]
impl<C, T, E> ContainerType for Edge<C, T, E>
where
    C: CursorType + Send + Sync,
    T: OutputType,
    E: ObjectType,
{
    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<Value>> {
        if ctx.item.node.name.node == "node" {
            let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
            return OutputType::resolve(&self.node, &ctx_obj, ctx.item)
                .await
                .map(Some);
        } else if ctx.item.node.name.node == "cursor" {
            return Ok(Some(Value::String(self.cursor.encode_cursor())));
        }

        self.additional_fields.resolve_field(ctx).await
    }
}

#[async_trait::async_trait]
impl<C, T, E> OutputType for Edge<C, T, E>
where
    C: CursorType + Send + Sync,
    T: OutputType,
    E: ObjectType,
{
    fn type_name() -> Cow<'static, str> {
        Cow::Owned(format!("{}Edge", T::type_name()))
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        registry.create_output_type::<Self, _>(|registry| {
            let additional_fields = if let registry::MetaType::Object { fields, .. } =
                registry.create_fake_output_type::<E>()
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
                        "cursor".to_string(),
                        registry::MetaField {
                            name: "cursor".to_string(),
                            description: Some("A cursor for use in pagination"),
                            args: Default::default(),
                            ty: String::create_type_info(registry),
                            deprecation: Default::default(),
                            cache_control: Default::default(),
                            external: false,
                            requires: None,
                            provides: None,
                            visible: None,
                            compute_complexity: None,
                        },
                    );

                    fields.extend(additional_fields);
                    fields
                },
                cache_control: Default::default(),
                extends: false,
                keys: None,
                visible: None,
                is_subscription: false,
                rust_typename: std::any::type_name::<Self>(),
            }
        })
    }

    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        _field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        resolve_container(ctx, self).await
    }
}

impl<C, T, E> ObjectType for Edge<C, T, E>
where
    C: CursorType + Send + Sync,
    T: OutputType,
    E: ObjectType,
{
}
