# Actix-web

## Request example

When you define your `actix_web::App` you need to pass in the Schema as data. 

```rust
async fn index(
    // Schema now accessible here
    schema: web::Data<Schema>,
    request: GraphQLRequest,
) -> web::Json<GraphQLResponse> {
    web::Json(schema.execute(request.into_inner()).await.into())
}
```

## Subscription example

```rust
async fn index_ws(
    schema: web::Data<Schema>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse> {
    GraphQLSubscription::new(Schema::clone(&*schema)).start(&req, payload)
}
```

## More examples

[https://github.com/async-graphql/examples/tree/master/actix-web](https://github.com/async-graphql/examples/tree/master/actix-web)
