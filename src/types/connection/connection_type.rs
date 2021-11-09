use std::borrow::Cow;

use futures_util::stream::{Stream, StreamExt, TryStreamExt};
use indexmap::map::IndexMap;

use crate::connection::edge::Edge;
use crate::connection::page_info::PageInfo;
use crate::parser::types::Field;
use crate::resolver_utils::{resolve_container, ContainerType};
use crate::types::connection::{CursorType, EmptyFields};
use crate::{
    registry, Context, ContextSelectionSet, ObjectType, OutputType, Positioned, Result,
    ServerResult, Value,
};

/// Connection type
///
/// Connection is the result of a query for `connection::query`.
pub struct Connection<C, T, EC = EmptyFields, EE = EmptyFields> {
    /// All edges of the current page.
    edges: Vec<Edge<C, T, EE>>,
    additional_fields: EC,
    has_previous_page: bool,
    has_next_page: bool,
}

impl<C, T, EE> Connection<C, T, EmptyFields, EE> {
    /// Create a new connection.
    pub fn new(has_previous_page: bool, has_next_page: bool) -> Self {
        Connection {
            additional_fields: EmptyFields,
            has_previous_page,
            has_next_page,
            edges: Vec::new(),
        }
    }
}

impl<C, T, EC, EE> Connection<C, T, EC, EE> {
    /// Create a new connection, it can have some additional fields.
    pub fn with_additional_fields(
        has_previous_page: bool,
        has_next_page: bool,
        additional_fields: EC,
    ) -> Self {
        Connection {
            additional_fields,
            has_previous_page,
            has_next_page,
            edges: Vec::new(),
        }
    }
}

impl<C, T, EC, EE> Connection<C, T, EC, EE> {
    /// Convert the edge type and return a new `Connection`.
    pub fn map<T2, EE2, F>(self, mut f: F) -> Connection<C, T2, EC, EE2>
    where
        F: FnMut(Edge<C, T, EE>) -> Edge<C, T2, EE2>,
    {
        let mut new_edges = Vec::with_capacity(self.edges.len());
        for edge in self.edges {
            new_edges.push(f(edge));
        }
        Connection {
            edges: new_edges,
            additional_fields: self.additional_fields,
            has_previous_page: self.has_previous_page,
            has_next_page: self.has_next_page,
        }
    }

    /// Convert the node type and return a new `Connection`.
    pub fn map_node<T2, F>(self, mut f: F) -> Connection<C, T2, EC, EE>
    where
        F: FnMut(T) -> T2,
    {
        self.map(|edge| Edge {
            cursor: edge.cursor,
            node: f(edge.node),
            additional_fields: edge.additional_fields,
        })
    }

    /// Append edges with `IntoIterator<Item = Edge<C, T, EE>>`
    pub fn append<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = Edge<C, T, EE>>,
    {
        self.edges.extend(iter);
    }

    /// Append edges with `IntoIterator<Item = Edge<C, T, EE>>`
    pub fn try_append<I>(&mut self, iter: I) -> Result<()>
    where
        I: IntoIterator<Item = Result<Edge<C, T, EE>>>,
    {
        for edge in iter {
            self.edges.push(edge?);
        }
        Ok(())
    }

    /// Append edges with `Stream<Item = Result<Edge<C, T, EE>>>`
    pub async fn append_stream<S>(&mut self, stream: S)
    where
        S: Stream<Item = Edge<C, T, EE>> + Unpin,
    {
        self.edges.extend(stream.collect::<Vec<_>>().await);
    }

    /// Append edges with `Stream<Item = Result<Edge<C, T, EE>>>`
    pub async fn try_append_stream<S>(&mut self, stream: S) -> Result<()>
    where
        S: Stream<Item = Result<Edge<C, T, EE>>> + Unpin,
    {
        self.edges.extend(stream.try_collect::<Vec<_>>().await?);
        Ok(())
    }
}

#[async_trait::async_trait]
impl<C, T, EC, EE> ContainerType for Connection<C, T, EC, EE>
where
    C: CursorType + Send + Sync,
    T: OutputType,
    EC: ObjectType,
    EE: ObjectType,
{
    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<Value>> {
        if ctx.item.node.name.node == "pageInfo" {
            let page_info = PageInfo {
                has_previous_page: self.has_previous_page,
                has_next_page: self.has_next_page,
                start_cursor: self.edges.first().map(|edge| edge.cursor.encode_cursor()),
                end_cursor: self.edges.last().map(|edge| edge.cursor.encode_cursor()),
            };
            let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
            return OutputType::resolve(&page_info, &ctx_obj, ctx.item)
                .await
                .map(Some);
        } else if ctx.item.node.name.node == "edges" {
            let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
            return OutputType::resolve(&self.edges, &ctx_obj, ctx.item)
                .await
                .map(Some);
        }

        self.additional_fields.resolve_field(ctx).await
    }
}

#[async_trait::async_trait]
impl<C, T, EC, EE> OutputType for Connection<C, T, EC, EE>
where
    C: CursorType + Send + Sync,
    T: OutputType,
    EC: ObjectType,
    EE: ObjectType,
{
    fn type_name() -> Cow<'static, str> {
        Cow::Owned(format!("{}Connection", T::type_name()))
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        registry.create_output_type::<Self, _>(|registry| {
            EC::create_type_info(registry);
            let additional_fields = if let Some(registry::MetaType::Object { fields, .. }) =
            registry.types.remove(EC::type_name().as_ref())
            {
                fields
            } else {
                unreachable!()
            };

            registry::MetaType::Object {
                name: Self::type_name().to_string(),
                description: None,
                fields: {
                    let mut fields = IndexMap::new();

                    fields.insert(
                        "pageInfo".to_string(),
                        registry::MetaField {
                            name: "pageInfo".to_string(),
                            description: Some("Information to aid in pagination."),
                            args: Default::default(),
                            ty: PageInfo::create_type_info(registry),
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
                        "edges".to_string(),
                        registry::MetaField {
                            name: "edges".to_string(),
                            description: Some("A list of edges."),
                            args: Default::default(),
                            ty: <Option<Vec<Option<Edge<C, T, EE>>>> as OutputType>::create_type_info(
                                registry,
                            ),
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

impl<C, T, EC, EE> ObjectType for Connection<C, T, EC, EE>
where
    C: CursorType + Send + Sync,
    T: OutputType,
    EC: ObjectType,
    EE: ObjectType,
{
}
