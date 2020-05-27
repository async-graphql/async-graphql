# Cursor connections

Relay's cursor connection specification is defined to provide a consistent method for query paging. For more details on the specification see the [GraphQL Cursor Connections Specification](https://facebook.github.io/relay/graphql/connections.htm)。

It is simple to define a cursor connection in `Async-GraphQL`

1. Implement `async_graphql::DataSource` and write the `execute_query` function.
2. Call `DataSource::query` in the field's resolver function and return the result.

Here is a simple data source that returns continuous integers:

```rust
use async_graphql::*;
use async_graphql::connection::*;

struct Integers;

#[DataSource]
impl DataSource for Integers {
    // Type for cursor
    type CursorType = usize;

    // Type for response
    type NodeType = i32;

    // We don't need to extend the connection fields, so this can be empty
    type ConnectionFieldsType = EmptyFields;

    // We don't need to extend the edge fields, so this can be empty
    type EdgeFieldsType = EmptyFields;

    async fn execute_query(
        &mut self, 
        _ctx: &Context<'_>, 
        after: Option<usize>, 
        before: Option<usize>, 
        first: Option<usize>, 
        last: Option<usize>,
    ) -> FieldResult<Connection<usize, i32, EmptyFields, EmptyFields>> {
        let mut start = after.map(|after| after + 1).unwrap_or(0);
        let mut end = before.unwrap_or(10000);
        if let Some(first) = first {
            end = (start + first).min(end);
        }
        if let Some(last) = last {
            start = if last > end - start {
                 end
            } else {
                end - last
            };
        }
        let mut connection = Connection::new(start > 0, end < 10000);
        connection.append(
            (start..end).into_iter().map(|n|
                Ok(Edge::new_with_additional_fields(n, n as i32, EmptyFields)),
        )?;
        Ok(connection)
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
    ) -> FieldResult<Connection<usize, i32, EmptyFields, EmptyFields>> {
        Integers.query(ctx, after, before, first, last).await
    }
}

```