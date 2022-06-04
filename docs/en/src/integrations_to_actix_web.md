# Actix-web

## Request example

When you define your `actix_web::App` you need to pass in the Schema as data. 

```rust
# extern crate async_graphql_actix_web;
# extern crate async_graphql;
# extern crate actix_web;
# use async_graphql::*;
# #[derive(Default,SimpleObject)]
# struct Query { a: i32 }
# let schema = Schema::build(Query::default(), EmptyMutation, EmptySubscription).finish();
use actix_web::{web, HttpRequest, HttpResponse};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
async fn index(
    // Schema now accessible here
    schema: web::Data<Schema<Query, EmptyMutation, EmptySubscription>>,
    request: GraphQLRequest,
) -> web::Json<GraphQLResponse> {
    web::Json(schema.execute(request.into_inner()).await.into())
}
```

## Subscription example

```rust
# extern crate async_graphql_actix_web;
# extern crate async_graphql;
# extern crate actix_web;
# use async_graphql::*;
# #[derive(Default,SimpleObject)]
# struct Query { a: i32 }
# let schema = Schema::build(Query::default(), EmptyMutation, EmptySubscription).finish();
use actix_web::{web, HttpRequest, HttpResponse};
use async_graphql_actix_web::GraphQLSubscription;
async fn index_ws(
    schema: web::Data<Schema<Query, EmptyMutation, EmptySubscription>>,
    req: HttpRequest,
    payload: web::Payload,
) -> actix_web::Result<HttpResponse> {
    GraphQLSubscription::new(Schema::clone(&*schema)).start(&req, payload)
}
```

## More examples

[https://github.com/async-graphql/examples/tree/master/actix-web](https://github.com/async-graphql/examples/tree/master/actix-web)
