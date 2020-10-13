# Introduction

`Async-graphql` is a GraphQL server-side library implemented in Rust. It is fully compatible with the GraphQL specification and most of its extensions, and offers type safety and high performance.

You can define a Schema in Rust and procedural macros will automatically generate code for a GraphQL query. This library does not extend Rust's syntax, which means that Rustfmt can be used normally. I value this highly and it is one of the reasons why I developed `Async-graphql`.

## Why do this?

I like GraphQL and Rust. I've been using `Juniper`, which solves the problem of implementing a GraphQL server with Rust. But Juniper had several problems, the most important of which is that it didn't support async/await at the time. So I decided to make this library for myself.

## Examples

If you are just getting started, we recommend checking out our examples at:
[https://github.com/async-graphql/examples](https://github.com/async-graphql/examples)

To see how you would create a Relay-compliant server using async-graphql, warp, diesel & postgresql, you can also check out a real-world example at:
[https://github.com/phated/twentyfive-stars](https://github.com/phated/twentyfive-stars)

## Benchmarks

Ensure that there is no CPU-heavy process in background!

```shell script
cd benchmark
cargo bench
```

Now a HTML report is available at `benchmark/target/criterion/report`.
