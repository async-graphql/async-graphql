# Introduction

'Async-graphql' is a GraphQL server-side library implemented in the Rust language. It is fully compatible with the GraphQL specification and most of its extensions, type-safe, and high-performance.

You can define the Schema in the Rust language, the procedural macros automatically generate the framework code for the GraphQL query, and the lack of extending Rust's syntax means that Rustfmt can be used normally, which I value very much, which is one of the reasons why I developed `Async-graphql`.

## Why do this?

I like GraphQL and Rust. I've been using `Juniper` before, which solved my problem of implementing the GraphQL server with Rust, but it has some regrets, the most important of which is that it didn't support async/await at the time, so I decided to make one for myself.

## Progress

As I write this document today (April 15, 2020), a month and a half after I started `Async-graphql` development, it has surpassed my goal of becoming a fully functional graphql server library.

