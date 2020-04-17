# Warp

`Async-graphql-warp`提供了两个`Filter`，`graphql`和`graphql_subscription`。

`graphql`用于执行`Query`和`Mutation`请求，他总是要求POST方法，输出一个包含`Schema`和`QueryBuilder的元组`，你可以在之后组合其它Filter，或者直接调用`QueryBuilder::execute`执行查询。

`graphql_subscription`用于实现基于Web Socket的订阅，它输出`warp::Reply`。

## 请求例子

```rust
let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
let filter = async_graphql_warp::graphql(schema).and_then(|(schema, builder): (_, QueryBuilder)| async move {
    // 执行查询
    let resp = builder.execute(&schema).await;

    // 返回结果
    Ok::<_, Infallible>(warp::reply::json(&GQLResponse(resp)).into_response())
});
warp::serve(filter).run(([0, 0, 0, 0], 8000)).await;
```

## 订阅例子

```rust
let schema = Schema::new(QueryRoot, EmptyMutation, SubscriptionRoot);
let filter = async_graphql_warp::graphql_subscription(schema);
warp::serve(filter).run(([0, 0, 0, 0], 8000)).await;
```
