# Cursor connections

Relay's cursor connection specification is designed to provide a consistent method for query paging. For more details on the specification see the [GraphQL Cursor Connections Specification](https://facebook.github.io/relay/graphql/connections.htm)。

Defining a cursor connection in `async-graphql` is very simple, you just call the `connection::query` function and query data in the closure.

```rust
use async_graphql::*;
use async_graphql::connection::*;

struct Query;

#[Object]
impl Query {
    #[field]
    async fn numbers(&self,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> FieldResult<Connection<usize, i32, EmptyFields, EmptyFields>> {
        query(after, before, first, last, |after, before, first, last| {
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
            ))?;
            Ok(connection)
        })
    }
}

```
