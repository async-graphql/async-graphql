use crate::types::connection::QueryOperation;
use crate::{Connection, Context, Cursor, DataSource, FieldResult, ObjectType};
use futures::{stream::BoxStream, StreamExt};
use std::collections::VecDeque;

/// You can use a Pin<Box<Stream<Item = (Cursor, E, T)>>> as a datasource
///
/// # Examples
///
/// ```rust
/// use async_graphql::*;
/// use byteorder::{ReadBytesExt, BE};
/// use futures::StreamExt;
///
/// struct QueryRoot;
///
/// #[Object]
/// impl QueryRoot {
///     async fn stream_connection(&self, ctx: &Context<'_>,
///         after: Option<Cursor>,
///         before: Option<Cursor>,
///         first: Option<i32>,
///         last: Option<i32>
///     ) -> FieldResult<Connection<&str>> {
///         let mut edges_stream = futures::stream::iter(vec!["a", "b", "c", "d", "e", "f"])
///             .map(|node| {
///                 let cursor: Cursor = node.to_owned().into();
///                 (cursor, EmptyEdgeFields, node)
///             })
///             .boxed();
///
///         edges_stream.query(ctx, after, before, first, last).await
///     }
/// }
///
/// #[async_std::main]
/// async fn main() {
///     let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
///
///     assert_eq!(
///         schema
///             .execute("{ streamConnection(first: 2) { edges { node } } }")
///             .await
///             .unwrap()
///             .data,
///         serde_json::json!({
///             "streamConnection": {
///                 "edges": [
///                     { "node": "a" },
///                     { "node": "b" }
///                 ]
///             },
///         })
///     );
///
///     assert_eq!(
///         schema
///             .execute("{ streamConnection(last: 2) { edges { node } } }")
///             .await
///             .unwrap()
///             .data,
///         serde_json::json!({
///             "streamConnection": {
///                 "edges": [
///                     { "node": "e" },
///                     { "node": "f" }
///                 ]
///             },
///         })
///     );
/// }
/// ```
#[async_trait::async_trait]
impl<'a, T, E> DataSource for BoxStream<'a, (Cursor, E, T)>
where
    T: Send + 'a,
    E: ObjectType + Send + 'a,
{
    type Element = T;
    type EdgeFieldsObj = E;

    async fn query_operation(
        &mut self,
        _ctx: &Context<'_>,
        operation: &QueryOperation,
    ) -> FieldResult<Connection<Self::Element, Self::EdgeFieldsObj>> {
        let mut count: usize = 0;
        let mut has_seen_before = false;
        let mut has_prev_page = false;
        let mut has_next_page = false;
        let mut edges = VecDeque::new();

        while let Some(edge) = self.next().await {
            count += 1;

            if has_seen_before {
                continue;
            }

            match operation {
                QueryOperation::After { after }
                | QueryOperation::Between { after, .. }
                | QueryOperation::FirstAfter { after, .. }
                | QueryOperation::FirstBetween { after, .. }
                | QueryOperation::LastAfter { after, .. }
                | QueryOperation::LastBetween { after, .. } => {
                    if *after == edge.0 {
                        has_prev_page = true;
                        has_next_page = false;
                        edges.clear();

                        continue;
                    }
                }
                _ => {}
            }

            match operation {
                QueryOperation::Before { before }
                | QueryOperation::Between { before, .. }
                | QueryOperation::FirstBefore { before, .. }
                | QueryOperation::FirstBetween { before, .. }
                | QueryOperation::LastBefore { before, .. }
                | QueryOperation::LastBetween { before, .. } => {
                    if *before == edge.0 {
                        has_seen_before = true;
                        has_next_page = true;

                        continue;
                    }
                }
                _ => {}
            }

            match operation {
                QueryOperation::First { limit }
                | QueryOperation::FirstAfter { limit, .. }
                | QueryOperation::FirstBefore { limit, .. }
                | QueryOperation::FirstBetween { limit, .. } => {
                    if edges.len() < *limit {
                        edges.push_back(edge)
                    } else {
                        has_next_page = true;
                    }
                }

                QueryOperation::Last { limit }
                | QueryOperation::LastAfter { limit, .. }
                | QueryOperation::LastBefore { limit, .. }
                | QueryOperation::LastBetween { limit, .. } => {
                    if edges.len() >= *limit {
                        has_prev_page = true;
                        edges.pop_front();
                    }
                    edges.push_back(edge);
                }

                _ => {
                    edges.push_back(edge);
                }
            }
        }

        Ok(Connection::new(
            Some(count),
            has_prev_page,
            has_next_page,
            edges.into(),
        ))
    }
}
