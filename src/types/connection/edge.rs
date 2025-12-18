use std::{borrow::Cow, marker::PhantomData};

use crate::{
    ComplexObject, ObjectType, OutputType, OutputTypeMarker, SimpleObject, TypeName,
    connection::{DefaultEdgeName, EmptyFields},
    types::connection::{CursorType, EdgeNameType},
};

/// An edge in a connection.
#[derive(SimpleObject)]
#[graphql(internal, name_type, shareable, complex)]
pub struct Edge<Cursor, Node, EdgeFields, Name = DefaultEdgeName>
where
    Cursor: CursorType + Send + Sync,
    Node: OutputType + OutputTypeMarker,
    EdgeFields: ObjectType,
    Name: EdgeNameType,
{
    #[graphql(skip)]
    _mark: PhantomData<Name>,
    /// A cursor for use in pagination
    #[graphql(skip)]
    pub cursor: Cursor,
    /// The item at the end of the edge
    pub node: Node,
    #[graphql(flatten)]
    pub(crate) additional_fields: EdgeFields,
}

#[ComplexObject(internal)]
impl<Cursor, Node, EdgeFields, Name> Edge<Cursor, Node, EdgeFields, Name>
where
    Cursor: CursorType + Send + Sync,
    Node: OutputType + OutputTypeMarker,
    EdgeFields: ObjectType,
    Name: EdgeNameType,
{
    /// A cursor for use in pagination
    async fn cursor(&self) -> String {
        self.cursor.encode_cursor()
    }
}

impl<Cursor, Node, EdgeFields, Name> TypeName for Edge<Cursor, Node, EdgeFields, Name>
where
    Cursor: CursorType + Send + Sync,
    Node: OutputType + OutputTypeMarker,
    EdgeFields: ObjectType,
    Name: EdgeNameType,
{
    #[inline]
    fn type_name() -> Cow<'static, str> {
        Name::type_name::<Node>().into()
    }
}

impl<Cursor, Node, EdgeFields, Name> Edge<Cursor, Node, EdgeFields, Name>
where
    Name: EdgeNameType,
    Cursor: CursorType + Send + Sync,
    Node: OutputType + OutputTypeMarker,
    EdgeFields: ObjectType,
{
    /// Create a new edge, it can have some additional fields.
    #[inline]
    pub fn with_additional_fields(
        cursor: Cursor,
        node: Node,
        additional_fields: EdgeFields,
    ) -> Self {
        Self {
            _mark: PhantomData,
            cursor,
            node,
            additional_fields,
        }
    }
}

impl<Cursor, Node, Name> Edge<Cursor, Node, EmptyFields, Name>
where
    Cursor: CursorType + Send + Sync,
    Node: OutputType + OutputTypeMarker,
    Name: EdgeNameType,
{
    /// Create a new edge.
    #[inline]
    pub fn new(cursor: Cursor, node: Node) -> Self {
        Self {
            _mark: PhantomData,
            cursor,
            node,
            additional_fields: EmptyFields,
        }
    }
}
