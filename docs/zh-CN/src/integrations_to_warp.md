# Warp

`Async-graphql-warp`提供了两个`Filter`，`graphql`和`graphql_subscription`。

`graphql`用于执行`Query`和`Mutation`请求，他总是要求POST方法，输出一个包含`async_graphql::Schema`和`async_graphql::Request的元组`，你可以在之后组合其它Filter，或者直接调用`Schema::execute`执行查询。

`graphql_subscription`用于实现基于Web Socket的订阅，它输出`warp::Reply`。

## 请求例子

```rust
type MySchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
let filter = async_graphql_warp::graphql(schema).and_then(|(schema, request): (MySchema, async_graphql::Request)| async move {
    // 执行查询
    let resp = schema.execute(request).await;

    // 返回结果
    Ok::<_, Infallible>(async_graphql_warp::Response::from(resp))
});
warp::serve(filter).run(([0, 0, 0, 0], 8000)).await;
```

## 订阅例子

```rust
let schema = Schema::new(QueryRoot, EmptyMutation, SubscriptionRoot);
let filter = async_graphql_warp::graphql_subscription(schema);
warp::serve(filter).run(([0, 0, 0, 0], 8000)).await;
```
