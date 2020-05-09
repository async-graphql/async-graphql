# Warp

For `Async-graphql-warp`, two `Filter` integrations are provided: `graphql` and `graphql_subscription`.

The `graphql` filter is used for execution `Query` and `Mutation` requests. It always asks for the POST method and outputs a `Schema` via `QueryBuilder`. You can combine other filters later, or directly call `QueryBuilder::execute` to execute the query.

`graphql_subscription` is used to implement WebSocket subscriptions. It outputs `warp::Reply`.

## Request example

```rust
let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
let filter = async_graphql_warp::graphql(schema).and_then(|(schema, builder): (_, QueryBuilder)| async move {
    // Execute query
    let resp = builder.execute(&schema).await;

    // Return result
    Ok::<_, Infallible>(warp::reply::json(&GQLResponse(resp)).into_response())
});
warp::serve(filter).run(([0, 0, 0, 0], 8000)).await;
```

## Subscription example

```rust
let schema = Schema::new(QueryRoot, EmptyMutation, SubscriptionRoot);
let filter = async_graphql_warp::graphql_subscription(schema);
warp::serve(filter).run(([0, 0, 0, 0], 8000)).await;
```
