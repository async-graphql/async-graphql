use std::{borrow::Cow, marker::PhantomData};

use crate::{
    connection::{edge::Edge, ConnectionNameType, EdgeNameType, PageInfo},
    types::connection::{CursorType, EmptyFields},
    Object, ObjectType, OutputType, TypeName,
};

/// Connection type
///
/// Connection is the result of a query for `connection::query`.
pub struct Connection<
    Name,
    EdgeName,
    Cursor,
    Node,
    ConnectionFields = EmptyFields,
    EdgeFields = EmptyFields,
> where
    Name: ConnectionNameType,
    EdgeName: EdgeNameType,
    Cursor: CursorType + Send + Sync,
    Node: OutputType,
    ConnectionFields: ObjectType,
    EdgeFields: ObjectType,
{
    _mark1: PhantomData<Name>,
    _mark2: PhantomData<EdgeName>,
    /// All edges of the current page.
    pub edges: Vec<Edge<EdgeName, Cursor, Node, EdgeFields>>,
    /// Additional fields for connection object.
    pub additional_fields: ConnectionFields,
    /// If `true` means has previous page.
    pub has_previous_page: bool,
    /// If `false` means has next page.
    pub has_next_page: bool,
}

impl<Name, EdgeName, Cursor, Node, EdgeFields>
    Connection<Name, EdgeName, Cursor, Node, EmptyFields, EdgeFields>
where
    Name: ConnectionNameType,
    EdgeName: EdgeNameType,
    Cursor: CursorType + Send + Sync,
    Node: OutputType,
    EdgeFields: ObjectType,
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

impl<Name, EdgeName, Cursor, Node, ConnectionFields, EdgeFields>
    Connection<Name, EdgeName, Cursor, Node, ConnectionFields, EdgeFields>
where
    Name: ConnectionNameType,
    EdgeName: EdgeNameType,
    Cursor: CursorType + Send + Sync,
    Node: OutputType,
    ConnectionFields: ObjectType,
    EdgeFields: ObjectType,
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
impl<Name, EdgeName, Cursor, Node, ConnectionFields, EdgeFields>
    Connection<Name, EdgeName, Cursor, Node, ConnectionFields, EdgeFields>
where
    Name: ConnectionNameType,
    EdgeName: EdgeNameType,
    Cursor: CursorType + Send + Sync,
    Node: OutputType,
    ConnectionFields: ObjectType,
    EdgeFields: ObjectType,
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
    async fn edges(&self) -> &[Edge<EdgeName, Cursor, Node, EdgeFields>] {
        &self.edges
    }

    #[graphql(flatten)]
    #[inline]
    async fn additional_fields(&self) -> &ConnectionFields {
        &self.additional_fields
    }
}

impl<Name, EdgeName, Cursor, Node, ConnectionFields, EdgeFields> TypeName
    for Connection<Name, EdgeName, Cursor, Node, ConnectionFields, EdgeFields>
where
    Name: ConnectionNameType,
    EdgeName: EdgeNameType,
    Cursor: CursorType + Send + Sync,
    Node: OutputType,
    ConnectionFields: ObjectType,
    EdgeFields: ObjectType,
{
    #[inline]
    fn type_name() -> Cow<'static, str> {
        Name::type_name::<Node>().into()
    }
}
