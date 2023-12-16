<div align="center">
<samp>

# async-graphql

**a high-performance graphql server library that's fully specification compliant**

</samp>

[Book](https://async-graphql.github.io/async-graphql/en/index.html) • [中文文档](https://async-graphql.github.io/async-graphql/zh-CN/index.html) • [Docs](https://docs.rs/async-graphql) • [GitHub repository](https://github.com/async-graphql/async-graphql) • [Cargo package](https://crates.io/crates/async-graphql)

---

![ci status](https://github.com/async-graphql/async-graphql/workflows/CI/badge.svg)
[![code coverage](https://codecov.io/gh/async-graphql/async-graphql/branch/master/graph/badge.svg)](https://codecov.io/gh/async-graphql/async-graphql/)
[![Unsafe Rust forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![Crates.io version](https://img.shields.io/crates/v/async-graphql.svg)](https://crates.io/crates/async-graphql)
[![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/async-graphql)
[![downloads](https://img.shields.io/crates/d/async-graphql.svg)](https://crates.io/crates/async-graphql)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/async-graphql/async-graphql/compare)

_This crate uses `#![forbid(unsafe_code)]` to ensure everything is implemented in 100% safe Rust._

</div>

## Static schema

```rs
use std::error::Error;

use async_graphql::{http::GraphiQLSource, EmptyMutation, EmptySubscription, Object, Schema};
use async_graphql_poem::*;
use poem::{listener::TcpListener, web::Html, *};

struct Query;

#[Object]
impl Query {
    async fn howdy(&self) -> &'static str {
        "partner"
    }
}

#[handler]
async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().finish())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // create the schema
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription).finish();

    // start the http server
    let app = Route::new().at("/", get(graphiql).post(GraphQL::new(schema)));
    println!("GraphiQL: http://localhost:8000");
    Server::new(TcpListener::bind("0.0.0.0:8000"))
        .run(app)
        .await?;
    Ok(())
}
```

## Dynamic schema
Requires the `dynamic-schema` feature to be enabled.

```rs
use std::error::Error;

use async_graphql::{dynamic::*, http::GraphiQLSource};
use async_graphql_poem::*;
use poem::{listener::TcpListener, web::Html, *};

#[handler]
async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().finish())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let query = Object::new("Query").field(Field::new(
        "howdy",
        TypeRef::named_nn(TypeRef::STRING),
        |_| FieldFuture::new(async { "partner" }),
    ));

    // create the schema
    let schema = Schema::build(query, None, None).register(query).finish()?;

    // start the http server
    let app = Route::new().at("/", get(graphiql).post(GraphQL::new(schema)));
    println!("GraphiQL: http://localhost:8000");
    Server::new(TcpListener::bind("0.0.0.0:8000"))
        .run(app)
        .await?;
    Ok(())
}
```

## Features

- Static and dynamic schemas are fully supported
- Fully supports async/await
- Type safety
- Rustfmt friendly (Procedural Macro)
- Custom scalars
- Minimal overhead
- Easy integration ([poem](https://crates.io/crates/poem), [axum](https://crates.io/crates/axum), [actix-web](https://crates.io/crates/actix-web), [tide](https://crates.io/crates/tide), [warp](https://crates.io/crates/warp), [rocket](https://crates.io/crates/rocket) ...)
- Upload files (Multipart request)
- Subscriptions (WebSocket transport)
- Custom extensions
- Error extensions
- Limit query complexity/depth
- Batch queries
- Apollo Persisted Queries
- Apollo Tracing extension
- Apollo Federation(v2)

> **Note**: Minimum supported Rust version: 1.74.0 or later

## Examples

All examples are in the [sub-repository](https://github.com/async-graphql/examples), located in the examples directory.

```shell
git submodule update # update the examples repo
cd examples && cargo run --bin [name]
```

For more information, see the [sub-repository](https://github.com/async-graphql/examples) README.md.

## Integrations

Integrations are what glue `async-graphql` with your web server, here are provided ones, or you can build your own!

- Poem [async-graphql-poem](https://crates.io/crates/async-graphql-poem)
- Actix-web [async-graphql-actix-web](https://crates.io/crates/async-graphql-actix-web)
- Warp [async-graphql-warp](https://crates.io/crates/async-graphql-warp)
- Tide [async-graphql-tide](https://crates.io/crates/async-graphql-tide)
- Rocket [async-graphql-rocket](https://github.com/async-graphql/async-graphql/tree/master/integrations/rocket)
- Axum [async-graphql-axum](https://github.com/async-graphql/async-graphql/tree/master/integrations/axum)

## Crate features

This crate offers the following features. Most are not activated by default, except the integrations of GraphiQL (`graphiql`) and GraphQL Playground (`playground`):

| feature                        | enables                                                                                                                                                                                       |
|:-------------------------------|:----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **`apollo_tracing`**           | Enable the [Apollo tracing extension](https://docs.rs/async-graphql/latest/async_graphql/extensions/struct.ApolloTracing.html).                                                               |
| **`apollo_persisted_queries`** | Enable the [Apollo persisted queries extension](https://docs.rs/async-graphql/latest/async_graphql/extensions/apollo_persisted_queries/struct.ApolloPersistedQueries.html).                   |
| **`log`**                      | Enable the [Logger extension](https://docs.rs/async-graphql/latest/async_graphql/extensions/struct.Logger.html).                                                                              |
| **`tracing`**                  | Enable the [Tracing extension](https://docs.rs/async-graphql/latest/async_graphql/extensions/struct.Tracing.html).                                                                            |
| **`opentelemetry`**            | Enable the [OpenTelemetry extension](https://docs.rs/async-graphql/latest/async_graphql/extensions/struct.OpenTelemetry.html).                                                                |
| **`unblock`**                  | Support [Asynchronous reader for Upload](types/struct.Upload.html)                                                                                                                            |
| **`bson`**                     | Integrate with the [`bson` crate](https://crates.io/crates/bson).                                                                                                                             |
| **`chrono`**                   | Integrate with the [`chrono` crate](https://crates.io/crates/chrono).                                                                                                                         |
| **`chrono-tz`**                | Integrate with the [`chrono-tz` crate](https://crates.io/crates/chrono-tz).                                                                                                                   |
| **`url`**                      | Integrate with the [`url` crate](https://crates.io/crates/url).                                                                                                                               |
| **`uuid`**                     | Integrate with the [`uuid` crate](https://crates.io/crates/uuid).                                                                                                                             |
| **`uuid08`**                   | Integrate with the [`uuid 0.8` crate](https://crates.io/crates/uuid/0.8.2).                                                                                                                   |
| **`string_number`**            | Enable the [StringNumber](types/struct.StringNumber.html).                                                                                                                                    |
| **`dataloader`**               | Support [DataLoader](dataloader/struct.DataLoader.html).                                                                                                                                      |
| **`secrecy`**                  | Integrate with the [`secrecy` crate](https://crates.io/crates/secrecy).                                                                                                                       |
| **`decimal`**                  | Integrate with the [`rust_decimal` crate](https://crates.io/crates/rust_decimal).                                                                                                             |
| **`bigdecimal`**               | Integrate with the [`bigdecimal` crate](https://crates.io/crates/bigdecimal).                                                                                                                 |
| **`cbor`**                     | Support for [serde_cbor](https://crates.io/crates/serde_cbor).                                                                                                                                |
| **`smol_str`**                 | Integrate with the [`smol_str` crate](https://crates.io/crates/smol_str).                                                                                                                     |
| **`hashbrown`**                | Integrate with the [`hashbrown` crate](https://github.com/rust-lang/hashbrown).                                                                                                               |
| **`time`**                     | Integrate with the [`time` crate](https://github.com/time-rs/time).                                                                                                                           |
| **`tokio-sync`**               | Integrate with the [`tokio::sync::RwLock`](https://docs.rs/tokio/1.18.1/tokio/sync/struct.RwLock.html) and [`tokio::sync::Mutex`](https://docs.rs/tokio/1.18.1/tokio/sync/struct.Mutex.html). |
| **`fast_chemail`**             | Integrate with the [`fast_chemail` crate](https://crates.io/crates/fast_chemail).                                                                                                             |
| **`tempfile`**                 | Save the uploaded content in the temporary file.                                                                                                                                              |
| **`dynamic-schema`**           | Support dynamic schema                                                                                                                                                                        |
| **`graphiql`**                 | Enables the [GraphiQL IDE](https://github.com/graphql/graphiql) integration                                                                                                                   |
| **`playground`**               | Enables the [GraphQL playground IDE](https://github.com/graphql/graphql-playground) integration                                                                                               |

### Observability

One of the tools used to monitor your graphql server in production is Apollo Studio. Apollo Studio is a cloud platform that helps you build, monitor, validate, and secure your organization's data graph.
Add the extension crate [`async_graphql_apollo_studio_extension`](https://github.com/async-graphql/async_graphql_apollo_studio_extension) to make this avaliable.

## Who's using `async-graphql` in production?

- [Vector](https://vector.dev/)
- [DiveDB](https://divedb.net)
- [Kairos Sports tech](https://kairostech.io/)
- [AxieInfinity](https://axieinfinity.com/)
- [Nando's](https://www.nandos.co.uk/)
- [Prima.it](https://www.prima.it/)
- [VoxJar](https://voxjar.com/)
- [Zenly](https://zen.ly/)
- [Brevz](https://brevz.io/)
- [thorndyke](https://www.thorndyke.ai/)
- [My Data My Consent](https://mydatamyconsent.com/)

## Community Showcase

- [rust-actix-graphql-sqlx-postgresql](https://github.com/camsjams/rust-actix-graphql-sqlx-postgresql)
  Using GraphQL with Rust and Apollo Federation
- [entity-rs](https://github.com/chipsenkbeil/entity-rs) A simplistic framework based on TAO, Facebook's distributed database for Social Graph.
- [vimwiki-server](https://github.com/chipsenkbeil/vimwiki-rs/tree/master/vimwiki-server) Provides graphql server to inspect and manipulate vimwiki files.
- [Diana](https://github.com/arctic-hen7/diana) Diana is a GraphQL system for Rust that's designed to work as simply as possible out of the box, without sacrificing configuration ability.
- [cindythink](https://www.cindythink.com/)
- [sudograph](https://github.com/sudograph/sudograph)

## Blog Posts

- [Async GraphQL with Rust](https://formidable.com/blog/2022/async-graphql-with-rust-1/)
- [GraphQL in Rust](https://romankudryashov.com/blog/2020/12/graphql-rust/)
- [How to implement a Rust micro-service using Rocket, GraphQL, PostgreSQL](https://lionkeng.medium.com/how-to-implement-a-rust-micro-service-using-rocket-graphql-postgresql-a3f455f2ae8b)
- [Running GraphQL on Lambda with Rust](https://dylananthony.com/posts/graphql-lambda-rust)

## References

- [GraphQL](https://graphql.org)
- [GraphQL Multipart Request](https://github.com/jaydenseric/graphql-multipart-request-spec)
- [Multipart HTTP protocol for GraphQL subscriptions](https://www.apollographql.com/docs/router/executing-operations/subscription-multipart-protocol/)
- [GraphQL Cursor Connections Specification](https://facebook.github.io/relay/graphql/connections.htm)
- [GraphQL over WebSocket Protocol](https://github.com/apollographql/subscriptions-transport-ws/blob/master/PROTOCOL.md)
- [Apollo Tracing](https://github.com/apollographql/apollo-tracing)
- [Apollo Federation](https://www.apollographql.com/docs/apollo-server/federation/introduction)

## License

Licensed under either of

- Apache License, Version 2.0,
  ([LICENSE-APACHE](./LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](./LICENSE-MIT) or http://opensource.org/licenses/MIT)
  at your option.
