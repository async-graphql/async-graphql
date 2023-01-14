# Warp

`Async-graphql-warp`提供了两个`Filter`，`graphql`和`graphql_subscription`。

`graphql`用于执行`Query`和`Mutation`请求，它提取 GraphQL 请求，然后输出一个包含`async_graphql::Schema`和`async_graphql::Request`元组，你可以在之后组合其它 Filter，或者直接调用`Schema::execute`执行查询。

`graphql_subscription`用于实现基于 Web Socket 的订阅，它输出`warp::Reply`。

## 请求例子

```rust
# extern crate async_graphql_warp;
# extern crate async_graphql;
# extern crate warp;
# use async_graphql::*;
# use std::convert::Infallible;
# use warp::Filter;
# struct QueryRoot;
# #[Object]
# impl QueryRoot { async fn version(&self) -> &str { "1.0" } }
# async fn other() {
type MySchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
let filter = async_graphql_warp::graphql(schema).and_then(|(schema, request): (MySchema, async_graphql::Request)| async move {
    // 执行查询
    let resp = schema.execute(request).await;

    // 返回结果
    Ok::<_, Infallible>(async_graphql_warp::GraphQLResponse::from(resp))
});
warp::serve(filter).run(([0, 0, 0, 0], 8000)).await;
# }
```

## 订阅例子

```rust
# extern crate async_graphql_warp;
# extern crate async_graphql;
# extern crate warp;
# use async_graphql::*;
# use futures_util::stream::{Stream, StreamExt};
# use std::convert::Infallible;
# use warp::Filter;
# struct SubscriptionRoot;
# #[Subscription]
# impl SubscriptionRoot {
#   async fn tick(&self) -> impl Stream<Item = i32> {
#     futures_util::stream::iter(0..10)
#   }
# }
# struct QueryRoot;
# #[Object]
# impl QueryRoot { async fn version(&self) -> &str { "1.0" } }
# async fn other() {
let schema = Schema::new(QueryRoot, EmptyMutation, SubscriptionRoot);
let filter = async_graphql_warp::graphql_subscription(schema);
warp::serve(filter).run(([0, 0, 0, 0], 8000)).await;
# }
```

## 更多例子

[https://github.com/async-graphql/examples/tree/master/warp](https://github.com/async-graphql/examples/tree/master/warp)
