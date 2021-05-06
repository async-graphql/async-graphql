# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

[Unreleased]

- If `InputObject` contains an unnamed field, the correct error message will be given. [#498](https://github.com/async-graphql/async-graphql/issues/498) 

## [2.8.4] 2021-04-23

- Fix the problem that the `ComplexObject` macro cannot work due to the `secret` attribute.

## [2.8.3] 2021-04-12

- Fixed an error in exporting Federation SDL.

## [2.8.2] 2021-04-09

- Now when the resolver returns the `Result` type, `E` can be all types that implement `async_graphql::Into<Error>`.

## [2.8.1] 2021-04-08

### Fixed

- Fix stack overflow during Registry::create_type for recursive type while running Schema::build. [#474](https://github.com/async-graphql/async-graphql/issues/474)

### Added

- Add `secret` attribute for arguments, they will not appear in the log.

```rust
#[Object]
impl Query {
    async fn login(&self, username:String, #[graphql(secret)] password: String) -> i32 {
        todo!()
    }
}
```

## [2.8.0] 2021-04-05

### Changed

- Rework `Extension`, now fully supports asynchronous, better to use than before, and can achieve more features, it contains a lot of changes. _(if you don't have a custom extension, it will not cause the existing code to fail to compile)_
  
### Added

- Add `async_graphql_warp::graphql_protocol`, `async_graphql_warp::graphql_subscription_upgrade` and `async_graphql_warp::graphql_subscription_upgrade_with_data` to control WebSocket subscription more finely. 

## [2.7.4] 2021-04-02

- Add the `BuildHasher` generic parameter to `dataloader::HashMapCache` to allow custom hashing algorithms. [#455](https://github.com/async-graphql/async-graphql/issues/455)

## [2.7.3] 2021-04-02
 
## Added 

- Add cache support for DataLoader. [#455](https://github.com/async-graphql/async-graphql/issues/455)
- Implements `ScalarType` for `serde_json::Value`.
- Add `SelectionField::alias` and `SelectionField::arguments` methods.

## Fixed  

- Prevent Warp WS Close, Ping, and Pong messages from being parsed as GraphQL [#459](https://github.com/async-graphql/async-graphql/pull/459)
- Fix Schema::sdl() does not include subscription definitions. [#464](https://github.com/async-graphql/async-graphql/issues/464)

## [2.7.2] 2021-04-01

## Removed

- Remove `SchemaBuilder::override_name` method. [#437](https://github.com/async-graphql/async-graphql/issues/437)
  
## Added

- Add `name` and `visible` attributes for `Newtype` macro for define a new scalar. [#437](https://github.com/async-graphql/async-graphql/issues/437)
- `NewType` macro now also implements `From<InnerType>` and `Into<InnerType>`.

## [2.7.1] 2021-03-31

- Add `Request::disable_introspection` method. [#456](https://github.com/async-graphql/async-graphql/issues/456)

## [2.7.0] 2021-03-27

## Fixed

- Fix chrono-tz integration. [#452](https://github.com/async-graphql/async-graphql/pull/452)

## Changed

- Rework Extension & TracingExtension & OpenTelemetryExtension

## [2.6.5] - 2021-03-24

- In websocket, if the client sends `start` before `connection_init`, the connection will be immediately disconnected and return `1011` error. [#451](https://github.com/async-graphql/async-graphql/issues/451)

## [2.6.4] - 2021-03-22

- Fix docs.

## [2.6.3] - 2021-03-22

### Added

- Add `extension::OpenTelemetry`.

### Removed

- Remove `TracingConfig`, now Request span always takes the current span as the parent, so this option is no longer needed.
- Remove `multipart` feature.

### Changed

- Now all features are not activated by default.

## [2.6.2] - 2021-03-20

- Add `SchemaBuilder::enable_subscription_in_federation` method.  [#449](https://github.com/async-graphql/async-graphql/issues/449)

## [2.6.1] - 2021-03-19

- Fix tracing extension doesn't work with async code. [#448](https://github.com/async-graphql/async-graphql/issues/448)

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
