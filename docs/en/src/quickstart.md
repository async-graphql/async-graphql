# Quickstart

## Add dependency libraries

```toml
[dependencies]
async-graphql = "1.18.0"
async-graphql-actix-web = "1.18.0" # If you need to integrate into actix-web
async-graphql-warp = "1.18.0" # If you need to integrate into warp
async-graphql-tide = "1.18.0" # If you need to integrate into tide
```

## Write a Schema

The Schema of a GraphQL contains a required Query, an optional Mutation, and an optional Subscription. These object types are described using the structure of the Rust language. The field of the structure corresponds to the field of the GraphQL object, but you need to mark it with `#[field]` so that the procedure macro provided by `Async-graphql` can correctly recognize it.

`Async-graphql` implements the mapping of common data types to GraphQL types, such as `i32`, `f64`, `Option<T>`, `Vec<T>`, etc. Also, you can [extend these base types](custom_scalars.md), which are called scalars in the GraphQL.

Here is a simple example where we provide just one query that returns the sum of `a` and `b`.

```rust
use async_graphql::*;

struct Query;

#[Object]
impl Query {
    #[field(desc = "Returns the sum of a and b")]
    async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }
}
```

## Execute the query

In our example, there is only a Query without a Mutation or Subscription, so we create the Schema with `EmptyMutation` and `EmptySubscription`, and then call `Schema::execute` to execute the Query.

```rust
let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
let res = schema.execute("{ add(a: 10, b: 20) }");
```

## Output the query results as JSON

`Schema::execute` returns `async_graphql::Result` with `async_graphql::http::GQLResponse` wrapped, and it can be directly converted to JSON.

```rust
let json = serde_json::to_string(&async_graphql::http::GQLResponse(res));
```

## Web server integration

Please refer to <https://github.com/async-graphql/examples>.
