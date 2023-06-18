# 游标连接 (Cursor Connections)

Relay 定义了一套游标连接规范，以提供一致性的分页查询方式，具体的规范文档请参考[GraphQL Cursor Connections Specification](https://facebook.github.io/relay/graphql/connections.htm)。

在`Async-graphql`中定义一个游标连接非常简单，你只需要调用 connection::query 函数，并在闭包中查询数据。

下面是一个简单的获取连续整数的数据源：

```rust
# extern crate async_graphql;
use async_graphql::*;
use async_graphql::types::connection::*;

struct Query;

#[Object]
impl Query {
    async fn numbers(&self,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<Connection<usize, i32, EmptyFields, EmptyFields>> {
        query(after, before, first, last, |after, before, first, last| async move {
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
            connection.edges.extend(
                (start..end).into_iter().map(|n|
                    Edge::with_additional_fields(n, n as i32, EmptyFields)
            ));
            Ok::<_, async_graphql::Error>(connection)
        }).await
    }
}

```
