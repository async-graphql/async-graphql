# Actix-web

`Async-graphql-actix-web`提供了`GraphQLRequest`提取器用于提取`GraphQL`请求，和`GraphQLResponse`用于输出`GraphQL`响应。

`GraphQLSubscription`用于创建一个支持 Web Socket 订阅的 Actor。

## 请求例子

你需要把 Schema 传入`actix_web::App`作为全局数据。

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
    schema: web::Data<Schema<Query, EmptyMutation, EmptySubscription>>,
    request: GraphQLRequest,
) -> web::Json<GraphQLResponse> {
    web::Json(schema.execute(request.into_inner()).await.into())
}
```

## 订阅例子

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

## 更多例子

[https://github.com/async-graphql/examples/tree/master/actix-web](https://github.com/async-graphql/examples/tree/master/actix-web)
