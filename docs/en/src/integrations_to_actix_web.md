# Actix-web

`Async-graphql-actix-web` provides an implementation of `actix_web::FromRequest` for `GQLRequest`. This is actually an abstraction around `GqlQueryBuilder` and you can call `GQLRequest::into_inner` to convert it into a `GqlQueryBuilder`。

`WSSubscription` is an Actor that supports WebSocket subscriptions。

## Request example

When you define your `actix_web::App` you need to pass in the `GqlSchema` as data. 

```rust
async fn index(
    // GqlSchema now accessible here
    schema: web::Data<GqlSchema>,
    gql_request: GQLRequest,
) -> web::Json<GQLResponse> {
    web::Json(GQLResponse(gql_request.into_inner().execute(&schema).await))
}

```

## Subscription example

```rust
async fn index_ws(
    schema: web::Data<GqlSchema>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse> {
    ws::start_with_protocols(WSSubscription::new(&schema), &["graphql-ws"], &req, payload)
}
```
