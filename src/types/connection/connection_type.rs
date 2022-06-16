use std::{borrow::Cow, marker::PhantomData};

use crate::{
    connection::{
        edge::Edge, ConnectionNameType, DefaultConnectionName, DefaultEdgeName, EdgeNameType,
        PageInfo,
    },
    types::connection::{CursorType, EmptyFields},
    Object, ObjectType, OutputType, TypeName,
};

/// Connection type
///
/// Connection is the result of a query for `connection::query`.
pub struct Connection<
    Cursor,
    Node,
    ConnectionFields = EmptyFields,
    EdgeFields = EmptyFields,
    Name = DefaultConnectionName,
    EdgeName = DefaultEdgeName,
> where
    Cursor: CursorType + Send + Sync,
    Node: OutputType,
    ConnectionFields: ObjectType,
    EdgeFields: ObjectType,
    Name: ConnectionNameType,
    EdgeName: EdgeNameType,
{
    _mark1: PhantomData<Name>,
    _mark2: PhantomData<EdgeName>,
    /// All edges of the current page.
    pub edges: Vec<Edge<Cursor, Node, EdgeFields, EdgeName>>,
    /// Additional fields for connection object.
    pub additional_fields: ConnectionFields,
    /// If `true` means has previous page.
    pub has_previous_page: bool,
    /// If `true` means has next page.
    pub has_next_page: bool,
}

impl<Cursor, Node, EdgeFields, Name, EdgeName>
    Connection<Cursor, Node, EmptyFields, EdgeFields, Name, EdgeName>
where
    Cursor: CursorType + Send + Sync,
    Node: OutputType,
    EdgeFields: ObjectType,
    Name: ConnectionNameType,
    EdgeName: EdgeNameType,
{
    /// Create a new connection.
    #[inline]
    pub fn new(has_previous_page: bool, has_next_page: bool) -> Self {
        Connection {
            _mark1: PhantomData,
            _mark2: PhantomData,
            additional_fields: EmptyFields,
            has_previous_page,
            has_next_page,
            edges: Vec::new(),
        }
    }
}

impl<Cursor, Node, ConnectionFields, EdgeFields, Name, EdgeName>
    Connection<Cursor, Node, ConnectionFields, EdgeFields, Name, EdgeName>
where
    Cursor: CursorType + Send + Sync,
    Node: OutputType,
    ConnectionFields: ObjectType,
    EdgeFields: ObjectType,
    Name: ConnectionNameType,
    EdgeName: EdgeNameType,
{
    /// Create a new connection, it can have some additional fields.
    #[inline]
    pub fn with_additional_fields(
        has_previous_page: bool,
        has_next_page: bool,
        additional_fields: ConnectionFields,
    ) -> Self {
        Connection {
            _mark1: PhantomData,
            _mark2: PhantomData,
            additional_fields,
            has_previous_page,
            has_next_page,
            edges: Vec::new(),
        }
    }
}

#[Object(internal, name_type)]
impl<Cursor, Node, ConnectionFields, EdgeFields, Name, EdgeName>
    Connection<Cursor, Node, ConnectionFields, EdgeFields, Name, EdgeName>
where
    Cursor: CursorType + Send + Sync,
    Node: OutputType,
    ConnectionFields: ObjectType,
    EdgeFields: ObjectType,
    Name: ConnectionNameType,
    EdgeName: EdgeNameType,
{
    /// Information to aid in pagination.
    async fn page_info(&self) -> PageInfo {
        PageInfo {
            has_previous_page: self.has_previous_page,
            has_next_page: self.has_next_page,
            start_cursor: self.edges.first().map(|edge| edge.cursor.0.encode_cursor()),
            end_cursor: self.edges.last().map(|edge| edge.cursor.0.encode_cursor()),
        }
    }

    /// A list of edges.
    #[inline]
    async fn edges(&self) -> &[Edge<Cursor, Node, EdgeFields, EdgeName>] {
        &self.edges
    }

    /// A list of nodes.
    async fn nodes(&self) -> Vec<&Node> {
        self.edges.iter().map(|e| &e.node).collect()
    }

    #[graphql(flatten)]
    #[inline]
    async fn additional_fields(&self) -> &ConnectionFields {
        &self.additional_fields
    }
}

impl<Cursor, Node, ConnectionFields, EdgeFields, Name, EdgeName> TypeName
    for Connection<Cursor, Node, ConnectionFields, EdgeFields, Name, EdgeName>
where
    Cursor: CursorType + Send + Sync,
    Node: OutputType,
    ConnectionFields: ObjectType,
    EdgeFields: ObjectType,
    Name: ConnectionNameType,
    EdgeName: EdgeNameType,
{
    #[inline]
    fn type_name() -> Cow<'static, str> {
        Name::type_name::<Node>().into()
    }
}
