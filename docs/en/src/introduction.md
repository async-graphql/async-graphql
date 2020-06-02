# Introduction

`Async-graphql` is a GraphQL server-side library implemented in Rust. It is fully compatible with the GraphQL specification and most of its extensions, and offers type safety and high performance.

You can define a Schema in Rust and procedural macros will automatically generate code for a GraphQL query. This library does not extend Rust's syntax, which means that Rustfmt can be used normally. I value this highly and it is one of the reasons why I developed `Async-graphql`.

## Why do this?

I like GraphQL and Rust. I've been using `Juniper`, which solves the problem of implementing a GraphQL server with Rust. But Juniper had several problems, the most important of which is that it didn't support async/await at the time. So I decided to make this library for myself.

## Stability: Unstable & Experimental

__This project doesn't currently follow [Semantic Versioning (SemVer)](https://semver.org/), and there can be breaking changes on any version numbers. We will begin following SemVer once the project reaches `v2.0.0`__

Even though this project is above `v1.0.0`, we are rapidly changing and improving the API. This has caused versioning problems that aren't easily resolved because the project became popular very quickly (it was only started in March 2020).

We currently plan to start following SemVer once we reach the `v2.0.0` release, which will happen once the API starts to stabilize. Unfortunately, we don't currently have the timeline for this.

In accordance with Rust's policy on pre-`1.0.0` crates, we will try to keep breaking changes limited to minor version changes, but if things don't seem to be compiling after an upgrade, it is likely you'll need to dive into compiler errors to update some syntax that changed. Feel free to open an [issue](https://github.com/async-graphql/async-graphql/issues) if something seems weird!

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

Now HTML report is available at `benchmark/target/criterion/report`
