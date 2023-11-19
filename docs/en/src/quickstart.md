# Quickstart

## Add dependency libraries

```toml
[dependencies]
async-graphql = "4.0"
async-graphql-actix-web = "4.0" # If you need to integrate into actix-web
async-graphql-warp = "4.0" # If you need to integrate into warp
async-graphql-tide = "4.0" # If you need to integrate into tide
```

## Write a Schema

The Schema of a GraphQL contains a required Query, an optional Mutation, and an optional Subscription. These object types are described using the structure of the Rust language. The field of the structure corresponds to the field of the GraphQL object.

`Async-graphql` implements the mapping of common data types to GraphQL types, such as `i32`, `f64`, `Option<T>`, `Vec<T>`, etc. Also, you can [extend these base types](custom_scalars.md), which are called scalars in the GraphQL.

Here is a simple example where we provide just one query that returns the sum of `a` and `b`.

```rust
# extern crate async_graphql;
use async_graphql::*;

struct Query;

#[Object]
impl Query {
    /// Returns the sum of a and b
    async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }
}
```

## Execute the query

In our example, there is only a Query without a Mutation or Subscription, so we create the Schema with `EmptyMutation` and `EmptySubscription`, and then call `Schema::execute` to execute the Query.

```rust
# extern crate async_graphql;
# use async_graphql::*;
#
# struct Query;
# #[Object]
# impl Query {
#   async fn version(&self) -> &str { "1.0" }    
# }
# async fn other() {
let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
let res = schema.execute("{ add(a: 10, b: 20) }").await;
# }
```

## Output the query results as JSON

```rust,ignore
let json = serde_json::to_string(&res);
```

## Web server integration
All examples are in the [sub-repository](https://github.com/async-graphql/examples), located in the examples directory.

```shell
git submodule update # update the examples repo
cd examples && cargo run --bin [name]
```

For more information, see the [sub-repository](https://github.com/async-graphql/examples) README.md.