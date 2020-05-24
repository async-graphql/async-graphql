//! Types for Relay-compliant server

mod connection_type;
mod cursor;
mod edge;
mod page_info;
mod slice;

use crate::{Context, FieldResult, ObjectType, OutputValueType};
pub use connection_type::{Connection, Record};
pub use cursor::CursorType;
pub use page_info::PageInfo;
use std::fmt::Display;

/// Empty edge extension object
#[async_graphql_derive::SimpleObject(internal)]
pub struct EmptyEdgeFields;

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
/// #[DataSource]
/// impl DataSource for Numbers {
///     type CursorType = usize;
///     type ElementType = i32;
///     type EdgeFieldsType = EmptyEdgeFields;
///
///     async fn execute_query(&self,
///         ctx: &Context<'_>,
///         after: Option<usize>,
///         before: Option<usize>,
///         first: Option<usize>,
///         last: Option<usize>,
///      ) -> FieldResult<Connection<Self::ElementType, Self::EdgeFieldsType>> {
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
///         Ok(Connection::new_from_iter(
///             (start..end).into_iter().map(|n| Record::new_without_edge_fields(n, n as i32)),
///             start > 0,
///             end < 10000,
///             Some(10000),
///         ))
///     }
/// }
///
/// #[Object]
/// impl QueryRoot {
///     async fn numbers(&self, ctx: &Context<'_>,
///         after: Option<String>,
///         before: Option<String>,
///         first: Option<i32>,
///         last: Option<i32>
///     ) -> FieldResult<Connection<i32>> {
///         Numbers.query(ctx, after, before, first, last).await
///     }
/// }
///
/// #[async_std::main]
/// async fn main() {
///     let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
///
///     assert_eq!(schema.execute("{ numbers(first: 2) { edges { node } } }").await.unwrap().data, serde_json::json!({
///         "numbers": {
///             "edges": [
///                 {"node": 0},
///                 {"node": 1}
///             ]
///         },
///     }));
///
///     assert_eq!(schema.execute("{ numbers(last: 2) { edges { node } } }").await.unwrap().data, serde_json::json!({
///         "numbers": {
///             "edges": [
///                 {"node": 9998},
///                 {"node": 9999}
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
    type ElementType: OutputValueType + Send;

    /// Fields for Edge
    ///
    /// Is a type that implements `ObjectType` and can be defined by the procedure macro `#[Object]`.
    type EdgeFieldsType: ObjectType + Send;

    /// Parses the parameters and executes the query.
    async fn query(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> FieldResult<Connection<Self::ElementType, Self::EdgeFieldsType>>
    where
        <Self::CursorType as CursorType>::DecodeError: Display + Send + Sync + 'static,
    {
        if first.is_some() && last.is_some() {
            return Err(
                "The \"first\" and \"last\" parameters cannot exist at the same time".into(),
            );
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
            Some(before) => Some(Self::CursorType::decode_cursor(&before)?),
            None => None,
        };

        let after = match after {
            Some(after) => Some(Self::CursorType::decode_cursor(&after)?),
            None => None,
        };

        self.execute_query(ctx, after, before, first, last).await
    }

    /// Execute query
    async fn execute_query(
        &self,
        ctx: &Context<'_>,
        after: Option<Self::CursorType>,
        before: Option<Self::CursorType>,
        first: Option<usize>,
        last: Option<usize>,
    ) -> FieldResult<Connection<Self::ElementType, Self::EdgeFieldsType>>;
}
