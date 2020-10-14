# Warp

For `Async-graphql-warp`, two `Filter` integrations are provided: `graphql` and `graphql_subscription`.

The `graphql` filter is used for execution `Query` and `Mutation` requests. It always asks for the POST method and outputs a `async_graphql::Schema` and `async_graphql::Request`.
You can combine other filters later, or directly call `Schema::execute` to execute the query.

`graphql_subscription` is used to implement WebSocket subscriptions. It outputs `warp::Reply`.

## Request example

```rust
type MySchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
let filter = async_graphql_warp::graphql(schema).and_then(|(schema, request): (MySchema, async_graphql::Request)| async move {
    // Execute query
    let resp = schema.execute(request).await;

    // Return result
    Ok::<_, Infallible>(async_graphql_warp::Response::from(resp))
});
warp::serve(filter).run(([0, 0, 0, 0], 8000)).await;
```

## Subscription example

```rust
let schema = Schema::new(QueryRoot, EmptyMutation, SubscriptionRoot);
let filter = async_graphql_warp::graphql_subscription(schema);
warp::serve(filter).run(([0, 0, 0, 0], 8000)).await;
```
