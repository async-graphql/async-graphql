# Apollo Tracing支持

`Apollo Tracing`提供了查询每个步骤的性能分析结果，它是一个`Schema`扩展，性能分析结果保存在`QueryResponse`中。

启用`Apollo Tracing`扩展需要在创建`Schema`的时候添加该扩展。

```rust
use async_graphql::*;
use async_graphql::extensions::ApolloTracing;

let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
    .extension(ApolloTracing) // 启用ApolloTracing扩展
    .finish();

```
