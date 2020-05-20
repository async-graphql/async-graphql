# Introduction

`Async-graphql` is a GraphQL server-side library implemented in Rust. It is fully compatible with the GraphQL specification and most of its extensions, and offers type safety and high performance.

You can define a Schema in Rust and procedural macros will automatically generate code for a GraphQL query. This library does not extend Rust's syntax, which means that Rustfmt can be used normally. I value this highly and it is one of the reasons why I developed `Async-graphql`.

## Why do this?

I like GraphQL and Rust. I've been using `Juniper`, which solves the problem of implementing a GraphQL server with Rust. But Juniper had several problems, the most important of which is that it didn't support async/await at the time. So I decided to make this library for myself.

## Progress

As I write this document today (April 15, 2020), a month and a half after I started `Async-graphql` development, it has surpassed my goal of becoming a fully functional graphql server library.


## Examples

If you are just getting started, we recommend checking out our examples at:
[https://github.com/async-graphql/examples](https://github.com/async-graphql/examples)

To see how you would create a Relay-compliant server using async-graphql, warp, diesel & postgresql, you can also check out a real-world example at:
[https://github.com/phated/twentyfive-stars](https://github.com/phated/twentyfive-stars)

## Benchmarks

```bash
git clone https://github.com/async-graphql/benchmark
cargo run --release
```
