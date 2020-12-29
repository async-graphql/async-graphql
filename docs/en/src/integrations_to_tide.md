# Actix-web

`async_graphql_tide` provides an implementation of [tide::Endpoint](https://docs.rs/tide/0.15.0/tide/trait.Endpoint.html) trait. It also provides `receive_request` and `respond` functions to convert a Tide request to a GraphQL request and back to Tide response, if you want to  handle the request manually.

## Request example

When you create your `tide` server, you will need to pass the `async_graphql_tide::endpoint` with your schema as the POST request handler. Please note that you need to enable the `attributes` feature in `async-std` for this example to work.

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

If you want to manually handle the request, for example to read a header, you can skip `async_graphql_tide::endpoint` and use `receive_request` and `respond` functions instead.

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
