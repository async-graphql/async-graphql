# Actix-web

`Async-graphql-actix-web`提供实现了`actix_web::FromRequest`的`GQLRequest`，它其实是QueryBuilder的包装，你可以调用`GQLRequest::into_inner`把它转换成一个`QueryBuilder`。

`WSSubscription`是一个支持Web Socket订阅的Actor。

## 请求例子

你需要把Schema传入`actix_web::App`作为全局数据。

```rust
async fn index(
    schema: web::Data<Schema>,
    gql_request: GQLRequest,
) -> web::Json<GQLResponse> {
    web::Json(GQLResponse(gql_request.into_inner().execute(&schema).await))
}

```

## 订阅例子

```rust
async fn index_ws(
    schema: web::Data<Schema>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse> {
    ws::start_with_protocols(WSSubscription::new(&schema), &["graphql-ws"], &req, payload)
}
```
