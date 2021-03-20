# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.6.2] - 2021-03-20

- Add `SchemaBuilder::enable_subscription_in_federation` method.  [#449](https://github.com/async-graphql/async-graphql/issues/449)

## [2.6.1] - 2021-03-19

Fix tracing extension doesn't work with async code. [#448](https://github.com/async-graphql/async-graphql/issues/448)

## [2.6.0] - 2021-03-18

- Add [ComplexObject](https://docs.rs/async-graphql/2.6.0/async_graphql/attr.ComplexObject.html) macro.

## [2.5.14] - 2021-03-14

- Add `DataLoader::loader` method. [#441](https://github.com/async-graphql/async-graphql/issues/441)
- Fix the validation does not work on some inline fragments.

## [2.5.13] - 2021-03-09

- Support generics in Subscription types. [#438](https://github.com/async-graphql/async-graphql/pull/438)

## [2.5.12] - 2021-03-09

- Remove unnecessary Box from WebSocket messages.
- Export subscription type to Federation SDL. (for [GraphGate](https://github.com/async-graphql/graphgate) 😁)
- Add `extends` attribute for derive macros Subscription and MergedSubscription.
- Add `SchemaBuilder::override_name` method. [#437](https://github.com/async-graphql/async-graphql/issues/437)

## [2.5.11] - 2021-03-07

- Execute `_entity` requests in parallel. [#431](https://github.com/async-graphql/async-graphql/issues/431)

## [2.5.10] - 2021-03-06

- Add descriptions for the exported Federation SDL.

## [2.5.9] - 2021-02-28

### Changed

- Moved `Variables` from `async_graphql::context::Variables` to `async_graphql::Variables`.

## [2.5.8] - 2021-02-27

### Added

- Allow the `deprecation` attribute to have no reason.

    ```rust
    #[derive(SimpleObject)]
    struct MyObject {
        #[graphql(deprecation)]
        a: i32,
    
        #[graphql(deprecation = true)]
        b: i32,
    
        #[graphql(deprecation = false)]
        c: i32,
    
        #[graphql(deprecation = "reason")]
        d: i32,
    }
    ```

## [2.5.7] - 2021-02-23

### Fixed

- Fix the problem that the borrowing lifetime returned by the `Context::data` function is too small.

## [2.5.6] - 2021-02-23

### Changed

- When introspection is disabled, introspection related types are no longer registered.

## [2.5.5] - 2021-02-22

### Added

- Add support for Federation [nested keys](https://www.apollographql.com/docs/federation/entities/#defining-a-compound-primary-key).

## [2.5.4] - 2021-02-15

### Fixed

- Fixed the error that the directive locations `FIELD_DEFINITION` and `ENUM_VALUE` cannot be parsed.

## [2.5.3] - 2021-02-13

### Fixed

- Fixed [#409](https://github.com/async-graphql/async-graphql/issues/409)

## [2.5.2] - 2021-02-06

### Added

- Add subscription support for tide with [tide-websockets](https://crates.io/crates/tide-websockets).

### Fixed

- Fixed the bug that can accept subscription requests during the initialization of WebSocket.
- Fixed GraphQL over WebSocket Protocol does not support ConnectionError events. [#406](https://github.com/async-graphql/async-graphql/issues/406)
