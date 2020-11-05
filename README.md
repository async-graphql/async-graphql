# A GraphQL server library implemented in Rust

<div align="center">
  <!-- CI -->
  <img src="https://github.com/async-graphql/async-graphql/workflows/CI/badge.svg" />
  <!-- codecov -->
  <img src="https://codecov.io/gh/async-graphql/async-graphql/branch/master/graph/badge.svg" />
  <!-- Crates version -->
  <a href="https://crates.io/crates/async-graphql">
    <img src="https://img.shields.io/crates/v/async-graphql.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/async-graphql">
    <img src="https://img.shields.io/crates/d/async-graphql.svg?style=flat-square"
      alt="Download" />
  </a>
  <!-- docs.rs docs -->
  <a href="https://docs.rs/async-graphql">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
  <a href="https://github.com/rust-secure-code/safety-dance/">
    <img src="https://img.shields.io/badge/unsafe-forbidden-success.svg?style=flat-square"
      alt="Unsafe Rust forbidden" />
  </a>
</div>

`Async-graphql` is a high-performance server-side library that supports all GraphQL specifications.

* [Feature Comparison](feature-comparison.md)
* [Book](https://async-graphql.github.io/async-graphql/en/index.html)
* [中文文档](https://async-graphql.github.io/async-graphql/zh-CN/index.html)
* [Docs](https://docs.rs/async-graphql)
* [GitHub repository](https://github.com/async-graphql/async-graphql)
* [Cargo package](https://crates.io/crates/async-graphql)
* Minimum supported Rust version: 1.46 or later

## Safety

This crate uses `#![forbid(unsafe_code)]` to ensure everything is implemented in 100% Safe Rust.

## Features

* Fully supports async/await
* Type safety
* Rustfmt friendly (Procedural Macro)
* Custom scalars
* Minimal overhead
* Easy integration (actix_web, tide, warp, rocket ...)
* Upload files (Multipart request)
* Subscriptions (WebSocket transport)
* Custom extensions
* Apollo Tracing extension
* Limit query complexity/depth
* Error Extensions
* Apollo Federation
* Batch Queries
* Apollo Persisted Queries

## Examples

All examples are in the [sub-repository](https://github.com/async-graphql/examples), located in the examples directory.

**Run an example:**

```shell
git submodule update # update the examples repo
cd examples && cargo run --bin [name]
```

## Benchmark

Ensure that there is no CPU-heavy process in background!

```shell script
cd benchmark

#measure all with system malloc
cargo bench

#measure only chat run
cargo bench -- "chat run"

#measure all with jemalloc
cargo bench --features jemalloc

#measure only simple run with jemalloc
cargo bench --features jemalloc -- "simple run"
```

Now HTML report is available at `benchmark/target/criterion/report`

Read more here: https://bheisler.github.io/criterion.rs/book/criterion_rs.html

## Integrations

* Actix-web [async-graphql-actix-web](https://crates.io/crates/async-graphql-actix-web)
* Warp [async-graphql-warp](https://crates.io/crates/async-graphql-warp)
* Tide [async-graphql-tide](https://crates.io/crates/async-graphql-tide)
* Rocket [async-graphql-rocket](https://github.com/async-graphql/async-graphql/tree/master/integrations/rocket)

## License

Licensed under either of

* Apache License, Version 2.0,
  (./LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license (./LICENSE-MIT or http://opensource.org/licenses/MIT)
  at your option.

## References

* [GraphQL](https://graphql.org)
* [GraphQL Multipart Request](https://github.com/jaydenseric/graphql-multipart-request-spec)
* [GraphQL Cursor Connections Specification](https://facebook.github.io/relay/graphql/connections.htm)
* [GraphQL over WebSocket Protocol](https://github.com/apollographql/subscriptions-transport-ws/blob/master/PROTOCOL.md)
* [Apollo Tracing](https://github.com/apollographql/apollo-tracing)
* [Apollo Federation](https://www.apollographql.com/docs/apollo-server/federation/introduction)

## Contribute

Welcome to contribute !
