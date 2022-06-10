# Context

The main goal of `Context` is to acquire global data attached to Schema and also data related to the actual query being processed.

## Store Data

Inside the `Context` you can put global data, like environment variables, db connection pool, whatever you may need in every query.

The data must implement `Send` and `Sync`.

You can request the data inside a query by just calling `ctx.data::<TypeOfYourData>()`.

**Note that if the return value of resolver function is borrowed from `Context`, you will need to explicitly state the lifetime of the argument.**

The following example shows how to borrow data in `Context`.

```rust
# extern crate async_graphql;
use async_graphql::*;

struct Query;

#[Object]
impl Query {
    async fn borrow_from_context_data<'ctx>(
        &self,
        ctx: &Context<'ctx>
    ) -> Result<&'ctx String> {
        ctx.data::<String>()
    }
}
```

### Schema data

You can put data inside the context at the creation of the schema, it's useful for data that do not change, like a connection pool.

An instance of how it would be written inside an application:

```rust
# extern crate async_graphql;
# use async_graphql::*;
# #[derive(Default,SimpleObject)]
# struct Query { version: i32}
# struct EnvStruct;
# let env_struct = EnvStruct;
# struct S3Object;
# let s3_storage = S3Object;
# struct DBConnection;
# let db_core = DBConnection;
let schema = Schema::build(Query::default(), EmptyMutation, EmptySubscription)
    .data(env_struct)
    .data(s3_storage)
    .data(db_core)
    .finish();
```

### Request data

You can put data inside the context at the execution of the request, it's useful for authentication data for instance.

A little example with a `warp` route:

```rust
# extern crate async_graphql;
# extern crate async_graphql_warp;
# extern crate warp;
# use async_graphql::*;
# use warp::{Filter, Reply};
# use std::convert::Infallible;
# #[derive(Default, SimpleObject)]
# struct Query { name: String }
# struct AuthInfo { pub token: Option<String> }
# let schema = Schema::build(Query::default(), EmptyMutation, EmptySubscription).finish();
# let schema_filter = async_graphql_warp::graphql(schema);
let graphql_post = warp::post()
  .and(warp::path("graphql"))
  .and(warp::header::optional("Authorization"))
  .and(schema_filter)
  .and_then( |auth: Option<String>, (schema, mut request): (Schema<Query, EmptyMutation, EmptySubscription>, async_graphql::Request)| async move {
    // Do something to get auth data from the header
    let your_auth_data = AuthInfo { token: auth };
    let response = schema
      .execute(
        request
         .data(your_auth_data)
      ).await;

    Ok::<_, Infallible>(async_graphql_warp::GraphQLResponse::from(response))
  });
```

## Headers

With the Context you can also insert and appends headers.

```rust
# extern crate async_graphql;
# extern crate http;
# use ::http::header::ACCESS_CONTROL_ALLOW_ORIGIN;
# use async_graphql::*;
# struct Query;
#[Object]
impl Query {
    async fn greet(&self, ctx: &Context<'_>) -> String {
        // Headers can be inserted using the `http` constants
        let was_in_headers = ctx.insert_http_header(ACCESS_CONTROL_ALLOW_ORIGIN, "*");

        // They can also be inserted using &str
        let was_in_headers = ctx.insert_http_header("Custom-Header", "1234");

        // If multiple headers with the same key are `inserted` then the most recent
        // one overwrites the previous. If you want multiple headers for the same key, use
        // `append_http_header` for subsequent headers
        let was_in_headers = ctx.append_http_header("Custom-Header", "Hello World");

        String::from("Hello world")
    }
}
```

## Selection / LookAhead

Sometimes you want to know what fields are requested in the subquery to optimize the processing of data. You can read fields accross the query with `ctx.field()` which will give you a `SelectionField` which will allow you to navigate accross the fields and subfields.

If you want to perform a search accross the query or the subqueries, you do not have to do this by hand with the `SelectionField`, you can use the `ctx.look_ahead()` to perform a selection

```rust
# extern crate async_graphql;
use async_graphql::*;

#[derive(SimpleObject)]
struct Detail {
    c: i32,
    d: i32,
}

#[derive(SimpleObject)]
struct MyObj {
    a: i32,
    b: i32,
    detail: Detail,
}

struct Query;

#[Object]
impl Query {
    async fn obj(&self, ctx: &Context<'_>) -> MyObj {
        if ctx.look_ahead().field("a").exists() {
            // This is a query like `obj { a }`
        } else if ctx.look_ahead().field("detail").field("c").exists() {
            // This is a query like `obj { detail { c } }`
        } else {
            // This query doesn't have `a`
        }
        unimplemented!()
    }
}
```
