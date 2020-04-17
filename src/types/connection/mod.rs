mod connection_type;
mod edge;
mod page_info;
mod slice;

use crate::{Context, FieldResult, ObjectType};

pub use connection_type::Connection;

/// Connection query operation
pub enum QueryOperation<'a> {
    /// Forward query
    Forward {
        /// After this cursor
        after: Option<&'a str>,

        /// How many records did this query return
        limit: usize,
    },
    /// Backward query
    Backward {
        /// Before this cursor
        before: Option<&'a str>,

        /// How many records did this query return
        limit: usize,
    },
}

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
/// use byteorder::{ReadBytesExt, BE};
///
/// struct QueryRoot;
///
/// #[SimpleObject]
/// struct DiffFields {
///     #[field]
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
///     async fn query_operation(&self, operation: &QueryOperation<'_>) -> FieldResult<Connection<Self::Element, Self::EdgeFieldsObj>> {
///         let (start, end) = match operation {
///             QueryOperation::Forward {after, limit} => {
///                 let start = after.and_then(|after| base64::decode(after).ok())
///                     .and_then(|data| data.as_slice().read_i32::<BE>().ok())
///                     .map(|idx| idx + 1)
///                     .unwrap_or(0);
///                 let end = start + *limit as i32;
///                 (start, end)
///             }
///             QueryOperation::Backward {before, limit} => {
///                 let end = before.and_then(|before| base64::decode(before).ok())
///                     .and_then(|data| data.as_slice().read_i32::<BE>().ok())
///                     .unwrap_or(0);
///                 let start = end - *limit as i32;
///                 (start, end)
///             }
///         };
///
///         let nodes = (start..end).into_iter().map(|n| (base64::encode(n.to_be_bytes()), DiffFields {diff: n - 1000}, n)).collect();
///         Ok(Connection::new(None, true, true, nodes))
///     }
/// }
///
/// #[Object]
/// impl QueryRoot {
///     #[field]
///     async fn numbers(&self, ctx: &Context<'_>,
///         after: Option<String>,
///         before: Option<String>,
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
pub trait DataSource: Sync + Send {
    /// Record type
    type Element;

    /// Fields for Edge
    ///
    /// Is a type that implements `ObjectType` and can be defined by the procedure macro `#[Object]`.
    type EdgeFieldsObj: ObjectType + Send + Sync;

    /// Execute the query.
    async fn query(
        &self,
        _ctx: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> FieldResult<Connection<Self::Element, Self::EdgeFieldsObj>> {
        let operation = if let Some(after) = &after {
            QueryOperation::Forward {
                after: Some(after),
                limit: match first {
                    Some(value) => value.max(0) as usize,
                    None => 10,
                },
            }
        } else if let Some(before) = &before {
            QueryOperation::Backward {
                before: Some(before),
                limit: match last {
                    Some(value) => value.max(0) as usize,
                    None => 10,
                },
            }
        } else if let Some(first) = first {
            QueryOperation::Forward {
                after: None,
                limit: first.max(0) as usize,
            }
        } else if let Some(last) = last {
            QueryOperation::Backward {
                before: None,
                limit: last.max(0) as usize,
            }
        } else {
            QueryOperation::Forward {
                after: None,
                limit: 10,
            }
        };

        self.query_operation(&operation).await
    }

    /// Parses the parameters and executes the query，Usually you just need to implement this method.
    async fn query_operation(
        &self,
        operation: &QueryOperation<'_>,
    ) -> FieldResult<Connection<Self::Element, Self::EdgeFieldsObj>>;
}
