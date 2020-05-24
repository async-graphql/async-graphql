use crate::connection::edge::Edge;
use crate::connection::page_info::PageInfo;
use crate::types::connection::{CursorType, EmptyEdgeFields};
use crate::{
    do_resolve, registry, Context, ContextSelectionSet, Error, FieldResult, ObjectType,
    OutputValueType, Positioned, QueryError, Result, Type,
};
use async_graphql_parser::query::Field;
use futures::{Stream, TryStreamExt};
use indexmap::map::IndexMap;
use inflector::Inflector;
use itertools::Itertools;
use std::borrow::Cow;

/// The record type output by the data source
pub struct Record<C, E, T> {
    cursor: C,
    edge: E,
    element: T,
}

impl<C, E, T> Record<C, E, T> {
    pub fn new(cursor: C, edge: E, element: T) -> Self {
        Self {
            cursor,
            edge,
            element,
        }
    }
}

impl<C, T> Record<C, EmptyEdgeFields, T> {
    pub fn new_without_edge_fields(cursor: C, element: T) -> Self {
        Self {
            cursor,
            edge: EmptyEdgeFields,
            element,
        }
    }
}

/// Connection type
///
/// Connection is the result of a query for `DataSource`.
pub struct Connection<T, E: ObjectType + Send = EmptyEdgeFields> {
    /// The total number of records.
    pub(crate) total_count: Option<usize>,

    /// Information about pagination in a connection.
    pub(crate) page_info: PageInfo,

    /// All records of the current page.
    pub(crate) nodes: Vec<Record<String, E, T>>,
}

impl<T: OutputValueType + Send, E: ObjectType + Send> Connection<T, E> {
    pub fn empty() -> Self {
        Connection {
            total_count: None,
            page_info: PageInfo {
                has_previous_page: false,
                has_next_page: false,
                start_cursor: None,
                end_cursor: None,
            },
            nodes: Vec::new(),
        }
    }

    pub fn new<C>(
        nodes: Vec<Record<C, E, T>>,
        has_previous_page: bool,
        has_next_page: bool,
        total_count: Option<usize>,
    ) -> Connection<T, E>
    where
        C: CursorType,
    {
        let nodes = nodes
            .into_iter()
            .map(|record| Record {
                cursor: record.cursor.encode_cursor(),
                edge: record.edge,
                element: record.element,
            })
            .collect_vec();
        Connection {
            total_count,
            page_info: PageInfo {
                has_previous_page,
                has_next_page,
                start_cursor: nodes.first().map(|record| record.cursor.clone()),
                end_cursor: nodes.last().map(|record| record.cursor.clone()),
            },
            nodes,
        }
    }

    pub async fn new_from_stream<C, S>(
        stream: S,
        has_previous_page: bool,
        has_next_page: bool,
        total_count: Option<usize>,
    ) -> FieldResult<Connection<T, E>>
    where
        C: CursorType,
        S: Stream<Item = FieldResult<Record<C, E, T>>> + Unpin,
    {
        Ok(Self::new(
            stream.try_collect::<Vec<_>>().await?,
            has_previous_page,
            has_next_page,
            total_count,
        ))
    }

    pub fn new_from_iter<C, I>(
        iter: I,
        has_previous_page: bool,
        has_next_page: bool,
        total_count: Option<usize>,
    ) -> Connection<T, E>
    where
        C: CursorType,
        I: IntoIterator<Item = Record<C, E, T>>,
    {
        Self::new(
            iter.into_iter().collect(),
            has_previous_page,
            has_next_page,
            total_count,
        )
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
            return OutputValueType::resolve(&self.page_info, &ctx_obj, ctx.item).await;
        } else if ctx.name.node == "edges" {
            let ctx_obj = ctx.with_selection_set(&ctx.selection_set);
            let edges = self
                .nodes
                .iter()
                .map(|record| {
                    Some(Edge {
                        cursor: &record.cursor,
                        extra_type: &record.edge,
                        node: &record.element,
                    })
                })
                .collect_vec();
            return OutputValueType::resolve(&edges, &ctx_obj, ctx.item).await;
        } else if ctx.name.node == "totalCount" {
            let ctx_obj = ctx.with_selection_set(&ctx.selection_set);
            return OutputValueType::resolve(
                &self.total_count.map(|n| n as i32),
                &ctx_obj,
                ctx.item,
            )
            .await;
        } else if ctx.name.node == T::type_name().to_plural().to_camel_case() {
            let ctx_obj = ctx.with_selection_set(&ctx.selection_set);
            let items = self
                .nodes
                .iter()
                .map(|record| &record.element)
                .collect_vec();
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
