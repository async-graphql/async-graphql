mod connection_type;
mod cursor;
mod edge;
mod page_info;
mod slice;
mod stream;

use crate::{Context, FieldResult, ObjectType};

pub use connection_type::Connection;
pub use cursor::Cursor;
pub use page_info::PageInfo;

/// Connection query operation
#[derive(Debug, Clone)]
pub enum QueryOperation {
    /// Return all results
    None,
    /// Return all results after the cursor
    After {
        /// After this cursor
        after: Cursor,
    },
    /// Return all results before the cursor
    Before {
        /// Before this cursor
        before: Cursor,
    },
    /// Return all results between the cursors
    Between {
        /// After this cursor
        after: Cursor,
        /// But before this cursor
        before: Cursor,
    },
    /// Return the amount of results specified by `limit`, starting from the beginning
    First {
        /// The maximum amount of results to return
        limit: usize,
    },
    /// Return the amount of results specified by `limit`, starting after the cursor
    FirstAfter {
        /// The maximum amount of results to return
        limit: usize,
        /// After this cursor
        after: Cursor,
    },
    /// Return the amount of results specified by `limit`, starting from the beginning but ending before the cursor
    FirstBefore {
        /// The maximum amount of results to return
        limit: usize,
        /// Before this cursor
        before: Cursor,
    },
    /// Return the amount of results specified by `limit`, but between the cursors. Limit includes beginning results.
    FirstBetween {
        /// The maximum amount of results to return
        limit: usize,
        /// After this cursor
        after: Cursor,
        /// But before this cursor
        before: Cursor,
    },
    /// Return the amount of results specified by `limit`, but before the end
    Last {
        /// The maximum amount of results to return
        limit: usize,
    },
    /// Return the amount of results specified by `limit`, but before the end. Must not include anything before the cursor.
    LastAfter {
        /// The maximum amount of results to return
        limit: usize,
        /// After this cursor
        after: Cursor,
    },
    /// Return the amount of results specified by `limit`, but before the cursor
    LastBefore {
        /// The maximum amount of results to return
        limit: usize,
        /// Before this cursor
        before: Cursor,
    },
    /// Return the amount of results specified by `limit`, but between the cursors. Limit includes ending results.
    LastBetween {
        /// The maximum amount of results to return
        limit: usize,
        /// After this cursor
        after: Cursor,
        /// But before this cursor
        before: Cursor,
    },
    /// An invalid query was made. For example: sending `first` and `last` in the same query
    Invalid,
}

/// Empty edge extension object
#[async_graphql_derive::SimpleObject(internal)]
pub struct EmptyEdgeFields;

// Temporary struct for to store values for pattern matching
struct Pagination {
    after: Option<Cursor>,
    before: Option<Cursor>,
    first: Option<i32>,
    last: Option<i32>,
}

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
/// use byteorder::{ReadBytesExt, BE};
///
/// struct QueryRoot;
///
/// #[SimpleObject]
/// struct DiffFields {
///     diff: i32,
/// }
///
/// struct Numbers;
///
/// #[DataSource]
/// impl DataSource for Numbers {
///     type Element = i32;
///     type EdgeFieldsObj = DiffFields;
///
///     async fn query_operation(&mut self, ctx: &Context<'_>, operation: &QueryOperation) -> FieldResult<Connection<Self::Element, Self::EdgeFieldsObj>> {
///         let (start, end) = match operation {
///             QueryOperation::First {limit} => {
///                 let start = 0;
///                 let end = start + *limit as i32;
///                 (start, end)
///             }
///             QueryOperation::Last {limit} => {
///                 let end = 0;
///                 let start = end - *limit as i32;
///                 (start, end)
///             }
///             QueryOperation::FirstAfter {after, limit} => {
///                 let start = base64::decode(after.to_string())
///                     .ok()
///                     .and_then(|data| data.as_slice().read_i32::<BE>().ok())
///                     .map(|idx| idx + 1)
///                     .unwrap_or(0);
///                 let end = start + *limit as i32;
///                 (start, end)
///             }
///             QueryOperation::LastBefore {before, limit} => {
///                 let end = base64::decode(before.to_string())
///                     .ok()
///                     .and_then(|data| data.as_slice().read_i32::<BE>().ok())
///                     .unwrap_or(0);
///                 let start = end - *limit as i32;
///                 (start, end)
///             }
///             // You should handle all cases instead of using a default like this
///             _ => (0, 10)
///         };
///
///         let nodes = (start..end).into_iter().map(|n| (base64::encode(n.to_be_bytes()).into(), DiffFields {diff: n - 1000}, n)).collect();
///         Ok(Connection::new(None, true, true, nodes))
///     }
/// }
///
/// #[Object]
/// impl QueryRoot {
///     async fn numbers(&self, ctx: &Context<'_>,
///         after: Option<Cursor>,
///         before: Option<Cursor>,
///         first: Option<i32>,
///         last: Option<i32>
///     ) -> FieldResult<Connection<i32, DiffFields>> {
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
///     assert_eq!(schema.execute("{ numbers(last: 2) { edges { node diff } } }").await.unwrap().data, serde_json::json!({
///         "numbers": {
///             "edges": [
///                 {"node": -2, "diff": -1002},
///                 {"node": -1, "diff": -1001}
///             ]
///         },
///     }));
/// }
/// ```
#[async_trait::async_trait]
pub trait DataSource: Send {
    /// Record type
    type Element;

    /// Fields for Edge
    ///
    /// Is a type that implements `ObjectType` and can be defined by the procedure macro `#[Object]`.
    type EdgeFieldsObj: ObjectType + Send;

    /// Execute the query.
    async fn query(
        &mut self,
        ctx: &Context<'_>,
        after: Option<Cursor>,
        before: Option<Cursor>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> FieldResult<Connection<Self::Element, Self::EdgeFieldsObj>> {
        let pagination = Pagination {
            first,
            last,
            before,
            after,
        };

        let operation = match pagination {
            // This is technically allowed according to the Relay Spec, but highly discouraged
            Pagination {
                first: Some(_),
                last: Some(_),
                before: _,
                after: _,
            } => QueryOperation::Invalid,
            Pagination {
                first: None,
                last: None,
                before: None,
                after: None,
            } => QueryOperation::None,
            Pagination {
                first: None,
                last: None,
                before: Some(before),
                after: None,
            } => QueryOperation::Before { before },
            Pagination {
                first: None,
                last: None,
                before: None,
                after: Some(after),
            } => QueryOperation::After { after },
            Pagination {
                first: None,
                last: None,
                before: Some(before),
                after: Some(after),
            } => QueryOperation::Between { after, before },
            Pagination {
                first: Some(limit),
                last: None,
                before: None,
                after: None,
            } => QueryOperation::First {
                limit: limit.max(0) as usize,
            },
            Pagination {
                first: Some(limit),
                last: None,
                before: Some(before),
                after: None,
            } => QueryOperation::FirstBefore {
                limit: limit.max(0) as usize,
                before,
            },
            Pagination {
                first: Some(limit),
                last: None,
                before: None,
                after: Some(after),
            } => QueryOperation::FirstAfter {
                limit: limit.max(0) as usize,
                after,
            },
            Pagination {
                first: Some(limit),
                last: None,
                before: Some(before),
                after: Some(after),
            } => QueryOperation::FirstBetween {
                limit: limit.max(0) as usize,
                after,
                before,
            },
            Pagination {
                first: None,
                last: Some(limit),
                before: None,
                after: None,
            } => QueryOperation::Last {
                limit: limit.max(0) as usize,
            },
            Pagination {
                first: None,
                last: Some(limit),
                before: Some(before),
                after: None,
            } => QueryOperation::LastBefore {
                limit: limit.max(0) as usize,
                before,
            },
            Pagination {
                first: None,
                last: Some(limit),
                before: None,
                after: Some(after),
            } => QueryOperation::LastAfter {
                limit: limit.max(0) as usize,
                after,
            },
            Pagination {
                first: None,
                last: Some(limit),
                before: Some(before),
                after: Some(after),
            } => QueryOperation::LastBetween {
                limit: limit.max(0) as usize,
                after,
                before,
            },
        };

        self.query_operation(ctx, &operation).await
    }

    /// Parses the parameters and executes the query，Usually you just need to implement this method.
    async fn query_operation(
        &mut self,
        ctx: &Context<'_>,
        operation: &QueryOperation,
    ) -> FieldResult<Connection<Self::Element, Self::EdgeFieldsObj>>;
}
