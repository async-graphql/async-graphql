# Actix-web

`async_graphql_tide` 提供一个 trait [tide::Endpoint](https://docs.rs/tide/0.15.0/tide/trait.Endpoint.html) 的实现。如果您想手动处理请求的话，它还提供了 `receive_request` 和 `respond` 函数来将一个 Tide 请求转换为一个 GraphQL 请求，并返回  `tide::Response`。

## Request example

当创建你的 `tide` 服务器时，你需要将 `Schema` 传递给 `async_graphql_tide::endpoint` 。请注意，你需要启用 `async-std` 中的 feature `attributes` 才能使这个例子正常工作。

```rust
use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject,
};
use tide::{http::mime, Body, Response, StatusCode};

#[derive(SimpleObject)]
pub struct Demo {
    pub id: usize,
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn demo(&self, _ctx: &Context<'_>) -> Demo {
        Demo { id: 42 }
    }
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut app = tide::new();

    // create schema
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

    // add tide endpoint
    app.at("/graphql")
        .post(async_graphql_tide::endpoint(schema));

    // enable graphql playground
    app.at("/").get(|_| async move {
        Ok(Response::builder(StatusCode::Ok)
            .body(Body::from_string(playground_source(
                // note that the playground needs to know
                // the path to the graphql endpoint
                GraphQLPlaygroundConfig::new("/graphql"),
            )))
            .content_type(mime::HTML)
            .build())
    });

    Ok(app.listen("127.0.0.1:8080").await?)
}
```

## Manually handle the request

如果你想手动处理请求，例如读取头，你可以跳过 `async_graphql_tide::endpoint` 而使用 `receive_request` 和 `respond` 函数。

```rust
app.at("/graphql").post(move |req: tide::Request<()>| {
    let schema = schema.clone();
    async move {
        let req = async_graphql_tide::receive_request(req).await?;
        async_graphql_tide::respond(schema.execute(req).await)
    }
});
```

## More examples

[https://github.com/async-graphql/examples/tree/master/tide](https://github.com/async-graphql/examples/tree/master/tide)
