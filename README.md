# The GraphQL server library implemented by rust 

<div align="center">
  <!-- CI -->
  <img src="https://github.com/sunli829/potatonet/workflows/CI/badge.svg" />
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

## Documentation

* [GitHub repository](https://github.com/sunli829/async-graphql)
* [Cargo package](https://crates.io/crates/async-graphql)
* Minimum supported Rust version: 1.39 or later

## Features

* Fully support async/await.
* Type safety.
* Rustfmt friendly (Procedural Macro).
* Custom scalar.
* Minimal overhead.
* Easy integration (hyper, actix_web, tide, rocket ...).

## Goals

- [ ] Types
    - [X] Scalar
        - [X] Integer
        - [X] Float
        - [X] String
        - [X] Bool
        - [X] ID
        - [X] DateTime
        - [X] UUID
    - [X] Complex Types
        - [X] List
        - [X] Non-Null
    - [X] Object
    - [X] Enum
    - [X] InputObject
    - [ ] Interface
    - [ ] Union
    - [ ] Generic Types
- [ ] Query
    - [X] Fields
    - [X] Arguments
    - [X] Alias
    - [ ] Fragments
    - [ ] Inline fragments
    - [X] Operation name
    - [X] Variables
    - [ ] Directives
        - [ ] @include
        - [ ] @skip
        - [ ] @deprecated
        - [ ] Custom Directive
    - [ ] Schema

## References

* [GraphQL](https://graphql.org)
