# 游标连接(Cursor Connections)

Relay定义了一套游标连接规范，以提供一致性的分页查询方式，具体的规范文档请参考[GraphQL Cursor Connections Specification](https://facebook.github.io/relay/graphql/connections.htm)。

在`Async-graphql`中定义一个连接非常简单，只需要两步：

1. 实现`async_graphql::DataSource`，重写`query_operation`函数。
2. 在字段的Resolver函数中调用`DataSource::query`函数并传递响应参数。

下面是一个简单的获取连续整数的数据源：

```rust
use async_graphql::*;

struct Integers;

#[DataSource]
impl DataSource for Integers {
    // 元素类型
    type Element = i32;

    // 我们不需要扩展边的字段，所以传EmptyEdgeFields
    type EdgeFieldsObj = EmptyEdgeFields;

    async fn query_operation(&mut self, _ctx: &Context<'_>, operation: &QueryOperation<'_>) -> FieldResult<Connection<Self::Element, Self::EdgeFieldsObj>> {
        let (start, end) = match operation {
            // 向前查找
            QueryOperation::First {limit} => {
                let start = 0;
                let end = start + *limit as i32;
                (start, end)
            }
            QueryOperation::FirstAfter {after, limit} => {
                // 起始数字，从after+1开始
                let start = after.parse::<i32>()
                    .ok()
                    .map(|after| after + 1)
                    .unwrap_or(0);
                (start, end + start + *limit)
            }
            // 向后查找
            QueryOperation::Last {limit} => {
                let end = 0;
                let start = end - *limit as i32;
                (start, end)
            }
            QueryOperation::LastBefore {before, limit} => {
                // 结束数字
                let end = before.parse::<i32>()
                    .ok()
                    .unwrap_or(0);
                (end - *limit, end)
            }
            // TODO: 建议处理所有条件
            _ => (0, 10)
        };

        // 创建节点，每个节点都是一个包含三个值的元组，依次是游标，扩展边对象，节点值
        let nodes = (start..end).into_iter().map(|n| (n.to_string(), EmptyEdgeFields, n)).collect();

        // 创建Connection并返回
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
        // 查询
        Integers.query(ctx, after, before, first, last).await
    }
}

```