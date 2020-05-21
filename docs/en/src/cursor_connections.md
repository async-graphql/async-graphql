# Cursor connections

Relay's cursor connection specification is defined to provide a consistent method for query paging. For more details on the specification see the [GraphQL Cursor Connections Specification](https://facebook.github.io/relay/graphql/connections.htm)。

It is simple to define a cursor connection in `Async-GraphQL`

1. Implement `async_graphql::DataSource` and write the `query_operation` function.
2. Call `DataSource::query` in the field's resolver function and return the result.

Here is a simple data source that returns continuous integers:

```rust
use async_graphql::*;

struct Integers;

#[DataSource]
impl DataSource for Integers {
    // Type for response
    type Element = i32;

    // We don't need to extend the edge fields, so this can be empty
    type EdgeFieldsObj = EmptyEdgeFields;

    async fn query_operation(&mut self, _ctx: &Context<'_>, operation: &QueryOperation<'_>) -> FieldResult<Connection<Self::Element, Self::EdgeFieldsObj>> {
        let (start, end) = match operation {
            // Look from beginning up to limit
            QueryOperation::First {limit} => {
                let start = 0;
                let end = start + *limit as i32;
                (start, end)
            }
            QueryOperation::FirstAfter {after, limit} => {
                // Look after number up to limit
                let start = after.parse::<i32>()
                    .ok()
                    .map(|after| after + 1)
                    .unwrap_or(0);
                (start, end + start + *limit)
            }
            // Look backward from last element up to limit
            QueryOperation::Last {limit} => {
                let end = 0;
                let start = end - *limit as i32;
                (start, end)
            }
            QueryOperation::LastBefore {before, limit} => {
                // Look before number up to limit
                let end = before.parse::<i32>()
                    .ok()
                    .unwrap_or(0);
                (end - *limit, end)
            }
            // TODO: Need to handle all conditions
            _ => (0, 10)
        };

        // Create nodes. Each node is a tuple containing three values: the cursor, extended edge object, and node value
        let nodes = (start..end).into_iter().map(|n| (n.to_string(), EmptyEdgeFields, n)).collect();

        Ok(Connection::new(None, true, true, nodes))
    }
}

struct Query;

#[Object]
impl Query {
    #[field]
    async fn numbers(&self,
        ctx: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> FieldResult<Connection<i32, EmptyEdgeFields>> {
        // Make the query
        Integers.query(ctx, after, before, first, last).await
    }
}

```