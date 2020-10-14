# Actix-web

`Async-graphql-actix-web`提供实现了`actix_web::FromRequest`的`Request`，它其实是`async_graphql::Request`的包装，你可以调用`Request::into_inner`把它转换成一个`async_graphql::Request`。

`WSSubscription`是一个支持Web Socket订阅的Actor。

## 请求例子

你需要把Schema传入`actix_web::App`作为全局数据。

```rust
async fn index(
    schema: web::Data<Schema>,
    request: async_graphql_actix_web::Request,
) -> web::Json<Response> {
    web::Json(Response(schema.execute(request.into_inner()).await)
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
