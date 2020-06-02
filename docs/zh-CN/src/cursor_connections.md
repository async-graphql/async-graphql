# 游标连接(Cursor Connections)

Relay定义了一套游标连接规范，以提供一致性的分页查询方式，具体的规范文档请参考[GraphQL Cursor Connections Specification](https://facebook.github.io/relay/graphql/connections.htm)。

在`Async-graphql`中定义一个连接非常简单，只需要两步：

1. 实现`async_graphql::DataSource`，重写`execute_query`函数。
2. 在字段的Resolver函数中调用`DataSource::query`函数并传递响应参数。

下面是一个简单的获取连续整数的数据源：

```rust
use async_graphql::*;
use async_graphql::connection::*;

struct Integers;

#[DataSource]
impl DataSource for Integers {
    // 游标类型
    type CursorType = usize;

    // 元素类型
    type NodeType = i32;

    // 我们不需要扩展连接的字段，所以传EmptyFields
    type ConnectionFieldsType = EmptyFields;

    // 我们不需要扩展边的字段，所以传EmptyFields
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
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> FieldResult<Connection<usize, i32, EmptyFields, EmptyFields>> {
        Integers.query(after, before, first, last).await
    }
}

```