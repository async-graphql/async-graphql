use std::{borrow::Cow, marker::PhantomData};

use crate::{
    connection::{DefaultEdgeName, EmptyFields},
    types::connection::{CursorType, EdgeNameType},
    InputValueError, InputValueResult, ObjectType, OutputType, Scalar, ScalarType, SimpleObject,
    TypeName, Value,
};

pub(crate) struct CursorScalar<T: CursorType>(pub(crate) T);

#[Scalar(internal, name = "String")]
impl<T: CursorType + Send + Sync> ScalarType for CursorScalar<T> {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => T::decode_cursor(&s)
                .map(Self)
                .map_err(InputValueError::custom),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        matches!(value, Value::String(_))
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.encode_cursor())
    }
}

/// An edge in a connection.
#[derive(SimpleObject)]
#[graphql(internal, name_type)]
pub struct Edge<Cursor, Node, EdgeFields, Name = DefaultEdgeName>
where
    Cursor: CursorType + Send + Sync,
    Node: OutputType,
    EdgeFields: ObjectType,
    Name: EdgeNameType,
{
    #[graphql(skip)]
    _mark: PhantomData<Name>,
    /// A cursor for use in pagination
    pub(crate) cursor: CursorScalar<Cursor>,
    /// The item at the end of the edge
    pub node: Node,
    #[graphql(flatten)]
    pub(crate) additional_fields: EdgeFields,
}

impl<Cursor, Node, EdgeFields, Name> TypeName for Edge<Cursor, Node, EdgeFields, Name>
where
    Cursor: CursorType + Send + Sync,
    Node: OutputType,
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
    Node: OutputType,
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
            cursor: CursorScalar(cursor),
            node,
            additional_fields,
        }
    }
}

impl<Cursor, Node, Name> Edge<Cursor, Node, EmptyFields, Name>
where
    Cursor: CursorType + Send + Sync,
    Node: OutputType,
    Name: EdgeNameType,
{
    /// Create a new edge.
    #[inline]
    pub fn new(cursor: Cursor, node: Node) -> Self {
        Self {
            _mark: PhantomData,
            cursor: CursorScalar(cursor),
            node,
            additional_fields: EmptyFields,
        }
    }
}
