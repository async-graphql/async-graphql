# Apollo Tracing 支持

`Apollo Tracing`提供了查询每个步骤的性能分析结果，它是一个`Schema`扩展，性能分析结果保存在`QueryResponse`中。

启用`Apollo Tracing`扩展需要在创建`Schema`的时候添加该扩展。

```rust
# extern crate async_graphql;
use async_graphql::*;
use async_graphql::extensions::ApolloTracing;

# struct Query;
# #[Object]
# impl Query { async fn version(&self) -> &str { "1.0" } }

let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
    .extension(ApolloTracing) // 启用 ApolloTracing 扩展
    .finish();

```
