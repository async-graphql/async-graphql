# Context

The main goal of `Context` is to acquire global data attached to Schema and also data related to the actual query being processed.

## Store Data

Inside the `Context` you can put global data, like environnement variables, db connection pool, whatever you may need in every query.

The data must implement `Send` and `Sync`.

You can request the data inside a query by just calling `ctx.data::<TypeOfYourData>()`.

**Note that if the return value of resolver function is borrowed from `Context`, you will need to explicitly state the lifetime of the argument.**

The following example shows how to borrow data in `Context`.

```rust
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

You can put data inside the context at the creation of the schema, it's usefull for data that do not change, like a connection pool.

An instance of how it would be written inside an application:
```rust
  let schema = Schema::build(Query::default(), Mutation::default(), EmptySubscription)
    .data(env_struct)
    .data(s3_storage)
    .data(db_core)
```

### Request data

You can put data inside the context at the execution of the request, it's usefull for authentification data for instance.

A little example with a `warp` route:

```rust
let graphql_post = warp::post()
  .and(warp::path("graphql"))
  .and(schema_filter)
  .and(a_warp_filter)
  ...
  .and_then( |schema: (Schema<Query, Mutation, Subscriptions>, async_graphql::Request), arg2: ArgType2 ...| async move {
    let (schema, request) = schema;
    let your_auth_data = auth_function_from_headers(headers).await?;
    let response = schema
      .execute(
        request
         .data(your_auth_data)
         .data(something_else)
      ).await;
      
    Ok(async_graphql_warp::Response::from(response))
  });
```


## Headers

With the Context you can also insert and appends headers.

```rust
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
        let was_in_headers = ctx.insert_http_header("Custom-Header", "Hello World");

        String::from("Hello world")
    }
}
```

## Selection / LookAhead

Sometimes you want to know what fields are requested in the subquery to optimize the processing of data. You can read fields accross the query with `ctx.fields()` which will give you a `SelectionField` which will allow you to navigate accross the fields and subfields.

If you want to perform a search accross the query or the subqueries, you do not have to do this by hand with the `SelectionField`, you can use the `ctx.look_ahead()` to perform a selection

```rust
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
