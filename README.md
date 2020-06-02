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
</div>

`Async-graphql` is a high-performance server-side library that supports all GraphQL specifications.

* [Feature Comparison](feature-comparison.md)
* [Book](https://async-graphql.github.io/async-graphql/en/index.html)
* [中文文档](https://async-graphql.github.io/async-graphql/zh-CN/index.html)
* [Docs](https://docs.rs/async-graphql)
* [GitHub repository](https://github.com/async-graphql/async-graphql)
* [Cargo package](https://crates.io/crates/async-graphql)
* Minimum supported Rust version: 1.42 or later

## Safety

This crate uses #![forbid(unsafe_code)] to ensure everything is implemented in 100% Safe Rust.

## Stability: Unstable & Experimental

__This project doesn't currently follow [Semantic Versioning (SemVer)](https://semver.org/), and there can be breaking changes on any version numbers. We will begin following SemVer once the project reaches `v2.0.0`__

Even though this project is above `v1.0.0`, we are rapidly changing and improving the API. This has caused versioning problems that aren't easily resolved because the project became popular very quickly (it was only started in March 2020).

We currently plan to start following SemVer once we reach the `v2.0.0` release, which will happen once the API starts to stabilize. Unfortunately, we don't currently have the timeline for this.

In accordance with Rust's policy on pre-`1.0.0` crates, we will try to keep breaking changes limited to minor version changes, but if things don't seem to be compiling after an upgrade, it is likely you'll need to dive into compiler errors to update some syntax that changed. Feel free to open an [issue](https://github.com/async-graphql/async-graphql/issues) if something seems weird!

## Examples

If you are just getting started, we recommend checking out our examples at: https://github.com/async-graphql/examples

To see how you would create a Relay-compliant server using async-graphql, warp, diesel & postgresql, you can also check out a real-world example at: https://github.com/phated/twentyfive-stars

## Benchmark

Ensure that there is no CPU-heavy process in background!

```shell script
cd benchmark
cargo bench
```

Now HTML report is available at `benchmark/target/criterion/report`

Read more here: https://bheisler.github.io/criterion.rs/book/criterion_rs.html

## Features

* Fully support async/await
* Type safety
* Rustfmt friendly (Procedural Macro)
* Custom scalar
* Minimal overhead
* Easy integration (hyper, actix_web, tide ...)
* Upload files (Multipart request)
* Subscription (WebSocket transport)
* Custom extension
* Apollo Tracing extension
* Limit query complexity/depth
* Error Extensions
* Apollo Federation

## Integrations

* Actix-web [async-graphql-actix-web](https://crates.io/crates/async-graphql-actix-web)
* Warp [async-graphql-warp](https://crates.io/crates/async-graphql-warp)
* Tide [async-graphql-tide](https://crates.io/crates/async-graphql-tide)

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
