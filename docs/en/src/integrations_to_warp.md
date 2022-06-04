# Warp

For `Async-graphql-warp`, two `Filter` integrations are provided: `graphql` and `graphql_subscription`.

The `graphql` filter is used for execution `Query` and `Mutation` requests. It extracts GraphQL request and outputs `async_graphql::Schema` and `async_graphql::Request`.
You can combine other filters later, or directly call `Schema::execute` to execute the query.

`graphql_subscription` is used to implement WebSocket subscriptions. It outputs `warp::Reply`.

## Request example

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
    // Execute query
    let resp = schema.execute(request).await;

    // Return result
    Ok::<_, Infallible>(async_graphql_warp::GraphQLResponse::from(resp))
});
warp::serve(filter).run(([0, 0, 0, 0], 8000)).await;
# }
```

## Subscription example

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

## More examples

[https://github.com/async-graphql/examples/tree/master/warp](https://github.com/async-graphql/examples/tree/master/warp)
