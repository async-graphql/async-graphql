use crate::types::connection::cursor::Cursor;
use crate::types::connection::edge::Edge;
use crate::types::connection::page_info::PageInfo;
use crate::{
    do_resolve, registry, Context, ContextSelectionSet, EmptyEdgeFields, Error, ObjectType,
    OutputValueType, Positioned, QueryError, Result, Type,
};
use async_graphql_parser::query::Field;
use indexmap::map::IndexMap;
use inflector::Inflector;
use itertools::Itertools;
use std::borrow::Cow;

/// Connection type
///
/// Connection is the result of a query for `DataSource`,
/// If the `T` type is `OutputValueType`, you can return the value as a field function directly,
/// otherwise you can use the `Connection::map` function to convert to a type that implements `OutputValueType`.
/// `E` is an extension object type that extends the edge fields.
pub struct Connection<T, E: ObjectType + Sync + Send = EmptyEdgeFields> {
    /// The total number of records.
    pub total_count: Option<usize>,

    /// Information about pagination in a connection.
    pub page_info: PageInfo,

    /// All records of the current page.
    pub nodes: Vec<(Cursor, E, T)>,
}

impl<T, E: ObjectType + Sync + Send> Connection<T, E> {
    /// Create a connection object.
    pub fn new(
        total_count: Option<usize>,
        has_previous_page: bool,
        has_next_page: bool,
        nodes: Vec<(Cursor, E, T)>,
    ) -> Self {
        Connection {
            total_count,
            page_info: PageInfo {
                has_previous_page,
                has_next_page,
                start_cursor: nodes.first().map(|(cursor, _, _)| cursor.clone()),
                end_cursor: nodes.last().map(|(cursor, _, _)| cursor.clone()),
            },
            nodes,
        }
    }

    /// Convert node type.
    pub fn map<O, F>(self, mut f: F) -> Connection<O, E>
    where
        F: FnMut(T) -> O,
    {
        Connection {
            total_count: self.total_count,
            page_info: self.page_info,
            nodes: self
                .nodes
                .into_iter()
                .map(|(cursor, edge_type, node)| (cursor, edge_type, f(node)))
                .collect(),
        }
    }

    #[doc(hidden)]
    #[inline]
    pub async fn page_info(&self) -> &PageInfo {
        &self.page_info
    }

    #[doc(hidden)]
    #[inline]
    pub async fn edges(&self) -> Option<Vec<Option<Edge<'_, T, E>>>> {
        Some(
            self.nodes
                .iter()
                .map(|(cursor, extra_type, node)| {
                    Some(Edge {
                        cursor,
                        extra_type,
                        node,
                    })
                })
                .collect_vec(),
        )
    }

    #[doc(hidden)]
    #[inline]
    pub async fn total_count(&self) -> Option<i32> {
        self.total_count.map(|n| n as i32)
    }
}

impl<T: OutputValueType + Send + Sync, E: ObjectType + Sync + Send> Type for Connection<T, E> {
    fn type_name() -> Cow<'static, str> {
        Cow::Owned(format!("{}Connection", T::type_name()))
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        registry.create_type::<Self, _>(|registry| registry::MetaType::Object {
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
                        deprecation: None,
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None
                    },
                );

                fields.insert(
                    "edges".to_string(),
                    registry::MetaField {
                        name: "edges".to_string(),
                        description: Some("A list of edges."),
                        args: Default::default(),
                        ty: <Option::<Vec<Option<Edge<T,E>>>> as Type>::create_type_info(registry),
                        deprecation: None,
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None
                    },
                );

                fields.insert(
                    "totalCount".to_string(),
                    registry::MetaField {
                        name: "totalCount".to_string(),
                        description: Some(r#"A count of the total number of objects in this connection, ignoring pagination. This allows a client to fetch the first five objects by passing "5" as the argument to "first", then fetch the total count so it could display "5 of 83", for example."#),
                        args: Default::default(),
                        ty: Option::<i32>::create_type_info(registry),
                        deprecation: None,
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None
                    },
                );

                let elements_name = T::type_name().to_plural().to_camel_case();
                fields.insert(elements_name.clone(),registry::MetaField {
                    name: elements_name,
                    description: Some(r#"A list of all of the objects returned in the connection. This is a convenience field provided for quickly exploring the API; rather than querying for "{ edges { node } }" when no edge data is needed, this field can be be used instead. Note that when clients like Relay need to fetch the "cursor" field on the edge to enable efficient pagination, this shortcut cannot be used, and the full "{ edges { node } }" version should be used instead."#),
                    args: Default::default(),
                    ty: Vec::<T>::type_name().to_string(),
                    deprecation: None,
                    cache_control: Default::default(),
                    external: false,
                    requires: None,
                    provides: None
                });

                fields
            },
            cache_control: Default::default(),
            extends: false,
            keys: None
        })
    }
}

#[async_trait::async_trait]
impl<T: OutputValueType + Send + Sync, E: ObjectType + Sync + Send> ObjectType
    for Connection<T, E>
{
    async fn resolve_field(&self, ctx: &Context<'_>) -> Result<serde_json::Value> {
        if ctx.name.node == "pageInfo" {
            let ctx_obj = ctx.with_selection_set(&ctx.selection_set);
            return OutputValueType::resolve(self.page_info().await, &ctx_obj, ctx.item).await;
        } else if ctx.name.node == "edges" {
            let ctx_obj = ctx.with_selection_set(&ctx.selection_set);
            return OutputValueType::resolve(&self.edges().await, &ctx_obj, ctx.item).await;
        } else if ctx.name.node == "totalCount" {
            let ctx_obj = ctx.with_selection_set(&ctx.selection_set);
            return OutputValueType::resolve(&self.total_count().await, &ctx_obj, ctx.item).await;
        } else if ctx.name.node == T::type_name().to_plural().to_camel_case() {
            let ctx_obj = ctx.with_selection_set(&ctx.selection_set);
            let items = self.nodes.iter().map(|(_, _, item)| item).collect_vec();
            return OutputValueType::resolve(&items, &ctx_obj, ctx.item).await;
        }

        Err(Error::Query {
            pos: ctx.position(),
            path: None,
            err: QueryError::FieldNotFound {
                field_name: ctx.name.to_string(),
                object: Connection::<T, E>::type_name().to_string(),
            },
        })
    }
}

#[async_trait::async_trait]
impl<T: OutputValueType + Send + Sync, E: ObjectType + Sync + Send> OutputValueType
    for Connection<T, E>
{
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        _field: &Positioned<Field>,
    ) -> Result<serde_json::Value> {
        do_resolve(ctx, self).await
    }
}
