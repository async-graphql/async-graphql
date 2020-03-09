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
* Easy integration (hyper, actix_web, tide ...).

## Goals

- [X] Types
    - [X] Scalar
        - [X] Integer
        - [X] Float
        - [X] String
        - [X] Bool
        - [X] ID
        - [X] DateTime
        - [X] UUID
    - [X] Containers 
        - [X] List
        - [X] Non-Null
    - [ ] Object
        - [X] Lifetime cycle   
    - [X] Enum
    - [X] InputObject
        - [X] Field default value
        - [X] Deprecated flag
    - [X] Interface
    - [X] Union
- [X] Query
    - [X] Fields
    - [X] Arguments
        - [X] Default value
        - [X] Deprecated flag
    - [X] Alias
    - [X] Fragments
    - [X] Inline fragments
    - [X] Operation name
    - [X] Variables
        - [X] Default value
        - [X] Parse value
    - [X] Directives
        - [X] @include
            - [X] FIELD
            - [X] FRAGMENT_SPREAD
            - [X] INLINE_FRAGMENT
        - [X] @skip
            - [X] FIELD
            - [X] FRAGMENT_SPREAD
            - [X] INLINE_FRAGMENT
    - [X] Schema
- [ ] Validation rules
    - [X] ArgumentsOfCorrectType
    - [X] DefaultValuesOfCorrectType
    - [X] FieldsOnCorrectType
    - [X] FragmentsOnCompositeTypes
    - [X] KnownArgumentNames
    - [ ] KnownDirectives
    - [X] KnownFragmentNames
    - [ ] KnownTypeNames
    - [ ] LoneAnonymousOperation
    - [X] NoFragmentCycles
    - [ ] NoUndefinedVariables
    - [ ] NoUnusedFragments
    - [ ] NoUnusedVariables
    - [ ] OverlappingFieldsCanBeMerged
    - [ ] PossibleFragmentSpreads
    - [ ] ProvidedNonNullArguments
    - [ ] ScalarLeafs
    - [ ] UniqueArgumentNames
    - [ ] UniqueFragmentNames
    - [ ] UniqueInputFieldNames
    - [ ] UniqueOperationNames
    - [ ] UniqueVariableNames
    - [ ] VariableInAllowedPosition
- [ ] Integration examples
    - [X] Actix-web
    - [ ] Hyper
    - [X] Tide

## License

Licensed under either of

* Apache License, Version 2.0,
  (./LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license (./LICENSE-MIT or http://opensource.org/licenses/MIT)
  at your option.

## References

* [GraphQL](https://graphql.org)
