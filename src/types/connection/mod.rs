//! Types for Relay-compliant server

mod connection_type;
mod cursor;
mod edge;
mod page_info;

use std::{fmt::Display, future::Future};

pub use connection_type::Connection;
pub use cursor::{CursorType, OpaqueCursor};
pub use edge::Edge;
pub use page_info::PageInfo;

use crate::{Error, ObjectType, OutputType, OutputTypeMarker, Result, SimpleObject};

/// Empty additional fields
#[derive(SimpleObject)]
#[graphql(internal, fake)]
pub struct EmptyFields;

/// Used to specify the edge name.
pub trait EdgeNameType: Send + Sync {
    /// Returns the edge type name.
    fn type_name<T: OutputTypeMarker>() -> String;
}

/// Name the edge type by default with the default format.
pub struct DefaultEdgeName;

impl EdgeNameType for DefaultEdgeName {
    fn type_name<T: OutputTypeMarker>() -> String {
        format!("{}Edge", T::type_name())
    }
}

/// Used to specify the connection name.
pub trait ConnectionNameType: Send + Sync {
    /// Returns the connection type name.
    fn type_name<T: OutputTypeMarker>() -> String;
}

/// Name the connection type by default with the default format.
pub struct DefaultConnectionName;

impl ConnectionNameType for DefaultConnectionName {
    fn type_name<T: OutputTypeMarker>() -> String {
        format!("{}Connection", T::type_name())
    }
}

mod private {
    pub trait NodesFieldSwitcher: Send + Sync {}

    impl NodesFieldSwitcher for super::DisableNodesField {}
    impl NodesFieldSwitcher for super::EnableNodesField {}
}

/// Allow switch if [`Connection`] contains `nodes` field in GQL output
///
/// This trait is sealed and can not be implemented outside of this crate.
pub trait NodesFieldSwitcherSealed: private::NodesFieldSwitcher {}

impl NodesFieldSwitcherSealed for DisableNodesField {}
impl NodesFieldSwitcherSealed for EnableNodesField {}

/// Enable (at compile time) `nodes` field in GQL output of [`Connection`]
pub struct EnableNodesField;

/// Disable (at compile time) `nodes` field in GQL output of [`Connection`]
pub struct DisableNodesField;

/// Parses the parameters and executes the query.
///
/// # Examples
///
/// ```rust
/// use std::borrow::Cow;
///
/// use async_graphql::*;
/// use async_graphql::types::connection::*;
///
/// struct Query;
///
/// struct Numbers;
///
/// #[derive(SimpleObject)]
/// struct Diff {
///     diff: i32,
/// }
///
/// #[Object]
/// impl Query {
///     async fn numbers(&self,
///         after: Option<String>,
///         before: Option<String>,
///         first: Option<i32>,
///         last: Option<i32>
///     ) -> Result<Connection<usize, i32, EmptyFields, Diff>> {
///         query(after, before, first, last, |after, before, first, last| async move {
///             let mut start = after.map(|after| after + 1).unwrap_or(0);
///             let mut end = before.unwrap_or(10000);
///             if let Some(first) = first {
///                 end = (start + first).min(end);
///             }
///             if let Some(last) = last {
///                 start = if last > end - start {
///                     end
///                 } else {
///                     end - last
///                 };
///             }
///             let mut connection = Connection::new(start > 0, end < 10000);
///             connection.edges.extend(
///                 (start..end).into_iter().map(|n|
///                     Edge::with_additional_fields(n, n as i32, Diff{ diff: (10000 - n) as i32 })),
///             );
///             Ok::<_, Error>(connection)
///         }).await
///     }
/// }
///
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
///
/// assert_eq!(schema.execute("{ numbers(first: 2) { edges { node diff } } }").await.into_result().unwrap().data, value!({
///     "numbers": {
///         "edges": [
///             {"node": 0, "diff": 10000},
///             {"node": 1, "diff": 9999},
///         ]
///     },
/// }));
///
/// assert_eq!(schema.execute("{ numbers(last: 2) { edges { node diff } } }").await.into_result().unwrap().data, value!({
///     "numbers": {
///         "edges": [
///             {"node": 9998, "diff": 2},
///             {"node": 9999, "diff": 1},
///         ]
///     },
/// }));
/// # });
/// ```
///
/// # Custom connection and edge type names
///
/// ```
/// use async_graphql::{connection::*, *};
///
/// #[derive(SimpleObject)]
/// struct MyObj {
///     a: i32,
///     b: String,
/// }
///
/// // Use to custom connection name
/// struct MyConnectionName;
///
/// impl ConnectionNameType for MyConnectionName {
///     fn type_name<T: OutputType>() -> String {
///         "MyConnection".to_string()
///     }
/// }
///
/// // Use to custom edge name
/// struct MyEdgeName;
///
/// impl EdgeNameType for MyEdgeName {
///     fn type_name<T: OutputType>() -> String {
///         "MyEdge".to_string()
///     }
/// }
///
/// struct Query;
///
/// #[Object]
/// impl Query {
///     async fn numbers(
///         &self,
///         after: Option<String>,
///         before: Option<String>,
///         first: Option<i32>,
///         last: Option<i32>,
///     ) -> Connection<usize, MyObj, EmptyFields, EmptyFields, MyConnectionName, MyEdgeName> {
///         let mut connection = Connection::new(false, false);
///         connection.edges.push(Edge::new(1, MyObj { a: 100, b: "abc".to_string() }));
///         connection
///     }
/// }
///
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
///
/// let query = r#"{
///     numbers(first: 2) {
///         __typename
///         edges { __typename node { a b } }
///     }
/// }"#;
/// let data = schema.execute(query).await.into_result().unwrap().data;
/// assert_eq!(data, value!({
///     "numbers": {
///         "__typename": "MyConnection",
///         "edges": [
///             {"__typename": "MyEdge", "node": { "a": 100, "b": "abc" }},
///         ]
///     },
/// }));
/// # });
/// ```
pub async fn query<
    Name,
    EdgeName,
    Cursor,
    Node,
    NodesVersion,
    ConnectionFields,
    EdgeFields,
    F,
    R,
    E,
>(
    after: Option<String>,
    before: Option<String>,
    first: Option<i32>,
    last: Option<i32>,
    f: F,
) -> Result<Connection<Cursor, Node, ConnectionFields, EdgeFields, Name, EdgeName, NodesVersion>>
where
    Name: ConnectionNameType,
    EdgeName: EdgeNameType,
    Cursor: CursorType + Send + Sync,
    <Cursor as CursorType>::Error: Display + Send + Sync + 'static,
    Node: OutputType + OutputTypeMarker,
    NodesVersion: NodesFieldSwitcherSealed,
    ConnectionFields: ObjectType,
    EdgeFields: ObjectType,
    F: FnOnce(Option<Cursor>, Option<Cursor>, Option<usize>, Option<usize>) -> R,
    R: Future<
        Output = Result<
            Connection<Cursor, Node, ConnectionFields, EdgeFields, Name, EdgeName, NodesVersion>,
            E,
        >,
    >,
    E: Into<Error>,
{
    query_with(after, before, first, last, f).await
}

/// Parses the parameters and executes the query and return a custom
/// `Connection` type.
///
/// `Connection<T>` and `Edge<T>` have certain limitations. For example, you
/// cannot customize the name of the type, so you can use this function to
/// execute the query and return a customized `Connection` type.
///
/// # Examples
///
/// ```rust
/// 
/// use async_graphql::*;
/// use async_graphql::types::connection::*;
///
/// #[derive(SimpleObject)]
/// struct MyEdge {
///     cursor: usize,
///     node: i32,
///     diff: i32,
/// }
///
/// #[derive(SimpleObject)]
/// struct MyConnection {
///     edges: Vec<MyEdge>,
///     page_info: PageInfo,
/// }
///
/// struct Query;
///
/// #[Object]
/// impl Query {
///     async fn numbers(&self,
///         after: Option<String>,
///         before: Option<String>,
///         first: Option<i32>,
///         last: Option<i32>
///     ) -> Result<MyConnection> {
///         query_with(after, before, first, last, |after, before, first, last| async move {
///             let mut start = after.map(|after| after + 1).unwrap_or(0);
///             let mut end = before.unwrap_or(10000);
///             if let Some(first) = first {
///                 end = (start + first).min(end);
///             }
///             if let Some(last) = last {
///                 start = if last > end - start {
///                     end
///                 } else {
///                     end - last
///                 };
///             }
///             let connection = MyConnection {
///                 edges: (start..end).into_iter().map(|n| MyEdge {
///                     cursor: n,
///                     node: n as i32,
///                     diff: (10000 - n) as i32,
///                 }).collect(),
///                 page_info: PageInfo {
///                     has_previous_page: start > 0,
///                     has_next_page: end < 10000,
///                     start_cursor: Some(start.encode_cursor()),
///                     end_cursor: Some(end.encode_cursor()),
///                 },
///             };
///             Ok::<_, Error>(connection)
///         }).await
///     }
/// }
///
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
///
/// assert_eq!(schema.execute("{ numbers(first: 2) { edges { node diff } } }").await.into_result().unwrap().data, value!({
///     "numbers": {
///         "edges": [
///             {"node": 0, "diff": 10000},
///             {"node": 1, "diff": 9999},
///         ]
///     },
/// }));
///
/// assert_eq!(schema.execute("{ numbers(last: 2) { edges { node diff } } }").await.into_result().unwrap().data, value!({
///     "numbers": {
///         "edges": [
///             {"node": 9998, "diff": 2},
///             {"node": 9999, "diff": 1},
///         ]
///     },
/// }));
/// # });
/// ```
pub async fn query_with<Cursor, T, F, R, E>(
    after: Option<String>,
    before: Option<String>,
    first: Option<i32>,
    last: Option<i32>,
    f: F,
) -> Result<T>
where
    Cursor: CursorType + Send + Sync,
    <Cursor as CursorType>::Error: Display + Send + Sync + 'static,
    F: FnOnce(Option<Cursor>, Option<Cursor>, Option<usize>, Option<usize>) -> R,
    R: Future<Output = Result<T, E>>,
    E: Into<Error>,
{
    let first = match first {
        Some(first) if first < 0 => {
            return Err(Error::new(
                "The \"first\" parameter must be a non-negative number",
            ));
        }
        Some(first) => Some(first as usize),
        None => None,
    };

    let last = match last {
        Some(last) if last < 0 => {
            return Err(Error::new(
                "The \"last\" parameter must be a non-negative number",
            ));
        }
        Some(last) => Some(last as usize),
        None => None,
    };

    let before = match before {
        Some(before) => Some(Cursor::decode_cursor(&before).map_err(Error::new_with_source)?),
        None => None,
    };

    let after = match after {
        Some(after) => Some(Cursor::decode_cursor(&after).map_err(Error::new_with_source)?),
        None => None,
    };

    f(after, before, first, last).await.map_err(Into::into)
}
