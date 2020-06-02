//! Types for Relay-compliant server

mod connection_type;
mod cursor;
mod edge;
mod page_info;
mod slice;

use crate::FieldResult;
pub use connection_type::Connection;
pub use cursor::CursorType;
pub use edge::Edge;
use futures::Future;
pub use page_info::PageInfo;
use std::fmt::Display;

/// Empty additional fields
#[async_graphql_derive::SimpleObject(internal)]
pub struct EmptyFields;

/// Data source of GraphQL Cursor Connections type
///
/// `Edge` is an extension object type that extends the edge fields, If you don't need it, you can use `EmptyEdgeFields`.
///
/// # References
/// (GraphQL Cursor Connections Specification)[https://facebook.github.io/relay/graphql/connections.htm]
///
/// # Examples
///
/// ```rust
/// use async_graphql::*;
/// use async_graphql::connection::*;
///
/// struct QueryRoot;
///
/// struct Numbers;
///
/// #[SimpleObject]
/// struct Diff {
///     diff: i32,
/// }
///
/// #[DataSource]
/// impl DataSource for Numbers {
///     type CursorType = usize;
///     type NodeType = i32;
///     type ConnectionFieldsType = EmptyFields;
///     type EdgeFieldsType = Diff;
///
///     async fn execute_query(&self,
///         after: Option<usize>,
///         before: Option<usize>,
///         first: Option<usize>,
///         last: Option<usize>,
///      ) -> FieldResult<Connection<Self::CursorType, Self::NodeType, Self::ConnectionFieldsType, Self::EdgeFieldsType>> {
///         let mut start = after.map(|after| after + 1).unwrap_or(0);
///         let mut end = before.unwrap_or(10000);
///         if let Some(first) = first {
///             end = (start + first).min(end);
///         }
///         if let Some(last) = last {
///             start = if last > end - start {
///                  end
///             } else {
///                 end - last
///             };
///         }
///         let mut connection = Connection::new(start > 0, end < 10000);
///         connection.append(
///             (start..end).into_iter().map(|n|
///                 Edge::with_additional_fields(n, n as i32, Diff{ diff: (10000 - n) as i32 })),
///         );
///         Ok(connection)
///     }
/// }
///
/// #[Object]
/// impl QueryRoot {
///     async fn numbers(&self,
///         after: Option<String>,
///         before: Option<String>,
///         first: Option<i32>,
///         last: Option<i32>
///     ) -> FieldResult<Connection<usize, i32, EmptyFields, Diff>> {
///         Numbers.query(after, before, first, last).await
///     }
/// }
///
/// #[async_std::main]
/// async fn main() {
///     let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
///
///     assert_eq!(schema.execute("{ numbers(first: 2) { edges { node diff } } }").await.unwrap().data, serde_json::json!({
///         "numbers": {
///             "edges": [
///                 {"node": 0, "diff": 10000},
///                 {"node": 1, "diff": 9999},
///             ]
///         },
///     }));
///
///     assert_eq!(schema.execute("{ numbers(last: 2) { edges { node diff } } }").await.unwrap().data, serde_json::json!({
///         "numbers": {
///             "edges": [
///                 {"node": 9998, "diff": 2},
///                 {"node": 9999, "diff": 1},
///             ]
///         },
///     }));
/// }
/// ```
#[async_trait::async_trait]
pub trait DataSource {
    /// Cursor type
    type CursorType: CursorType + Send + Sync;

    /// Record type
    type NodeType;

    /// Additional fields for connection
    ///
    /// Is a type that implements `ObjectType` and can be defined by the procedure macro `#[Object]` or `#[SimpleObject]`.
    ///
    type ConnectionFieldsType;

    /// Additional fields for edge
    ///
    /// Is a type that implements `ObjectType` and can be defined by the procedure macro `#[Object]` or `#[SimpleObject]`.
    ///
    type EdgeFieldsType;

    /// Parses the parameters and executes the query.
    async fn query(
        &self,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> FieldResult<
        Connection<
            Self::CursorType,
            Self::NodeType,
            Self::ConnectionFieldsType,
            Self::EdgeFieldsType,
        >,
    >
    where
        <Self::CursorType as CursorType>::Error: Display + Send + Sync + 'static,
    {
        query(after, before, first, last, |after, before, first, last| {
            self.execute_query(after, before, first, last)
        })
        .await
    }

    /// Execute query
    async fn execute_query(
        &self,
        after: Option<Self::CursorType>,
        before: Option<Self::CursorType>,
        first: Option<usize>,
        last: Option<usize>,
    ) -> FieldResult<
        Connection<
            Self::CursorType,
            Self::NodeType,
            Self::ConnectionFieldsType,
            Self::EdgeFieldsType,
        >,
    >;
}

/// Parses the parameters and executes the query.
///
/// If you don't want to implement `DataSource`, you can also use this function to query data directly.
///
/// # Examples
///
/// ```rust
/// use async_graphql::*;
/// use async_graphql::connection::*;
///
/// struct QueryRoot;
///
/// struct Numbers;
///
/// #[SimpleObject]
/// struct Diff {
///     diff: i32,
/// }
///
/// #[Object]
/// impl QueryRoot {
///     async fn numbers(&self,
///         after: Option<String>,
///         before: Option<String>,
///         first: Option<i32>,
///         last: Option<i32>
///     ) -> FieldResult<Connection<usize, i32, EmptyFields, Diff>> {
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
///             connection.append(
///                 (start..end).into_iter().map(|n|
///                     Edge::with_additional_fields(n, n as i32, Diff{ diff: (10000 - n) as i32 })),
///             );
///             Ok(connection)
///         }).await
///     }
/// }
///
/// #[async_std::main]
/// async fn main() {
///     let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
///
///     assert_eq!(schema.execute("{ numbers(first: 2) { edges { node diff } } }").await.unwrap().data, serde_json::json!({
///         "numbers": {
///             "edges": [
///                 {"node": 0, "diff": 10000},
///                 {"node": 1, "diff": 9999},
///             ]
///         },
///     }));
///
///     assert_eq!(schema.execute("{ numbers(last: 2) { edges { node diff } } }").await.unwrap().data, serde_json::json!({
///         "numbers": {
///             "edges": [
///                 {"node": 9998, "diff": 2},
///                 {"node": 9999, "diff": 1},
///             ]
///         },
///     }));
/// }
/// ```
pub async fn query<Cursor, Node, ConnectionFields, EdgeFields, F, R>(
    after: Option<String>,
    before: Option<String>,
    first: Option<i32>,
    last: Option<i32>,
    mut f: F,
) -> FieldResult<Connection<Cursor, Node, ConnectionFields, EdgeFields>>
where
    Cursor: CursorType + Send + Sync,
    <Cursor as CursorType>::Error: Display + Send + Sync + 'static,
    F: FnMut(Option<Cursor>, Option<Cursor>, Option<usize>, Option<usize>) -> R,
    R: Future<Output = FieldResult<Connection<Cursor, Node, ConnectionFields, EdgeFields>>>,
{
    if first.is_some() && last.is_some() {
        return Err("The \"first\" and \"last\" parameters cannot exist at the same time".into());
    }

    let first = match first {
        Some(first) if first < 0 => {
            return Err("The \"first\" parameter must be a non-negative number".into())
        }
        Some(first) => Some(first as usize),
        None => None,
    };

    let last = match last {
        Some(last) if last < 0 => {
            return Err("The \"last\" parameter must be a non-negative number".into())
        }
        Some(last) => Some(last as usize),
        None => None,
    };

    let before = match before {
        Some(before) => Some(Cursor::decode_cursor(&before)?),
        None => None,
    };

    let after = match after {
        Some(after) => Some(Cursor::decode_cursor(&after)?),
        None => None,
    };

    f(after, before, first, last).await
}
