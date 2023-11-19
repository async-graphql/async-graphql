# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

# [6.0.11] 2023-11-19

- Clean up example docs [#1411](https://github.com/async-graphql/async-graphql/pull/1411)
- Run batch requests concurrently [#1420](https://github.com/async-graphql/async-graphql/pull/1420)
- Update opentelemetry to `v0.21.x` [#1422](https://github.com/async-graphql/async-graphql/pull/1422)

# [6.0.10] 2023-11-04

- bump opentelemetry `0.20.0` [#1406](https://github.com/async-graphql/async-graphql/pull/1406)
- fix check for serial [#1405](https://github.com/async-graphql/async-graphql/pull/1405)
- fixes complexity visitor
- bump Rocket from `0.5.0-rc.2` to `0.5.0-rc.4`

# [6.0.9] 2023-10-21

- add support uploading files in dynamic schema [#1384](https://github.com/async-graphql/async-graphql/discussions/1384)
- Include `@composeDirective` in Federation's `_service` field and document `#[TypeDirective]` [#1400](https://github.com/async-graphql/async-graphql/pull/1400)

# [6.0.7] 2023-09-23

- initialize source field in tracing extension parse_query method [#1367](https://github.com/async-graphql/async-graphql/pull/1367)
- test(variables): empty object passes but empty array fails [#1377](https://github.com/async-graphql/async-graphql/pull/1377)
- Add support for entities without a reference resolver [#1378](https://github.com/async-graphql/async-graphql/pull/1378)
- Fixes [#1356](https://github.com/async-graphql/async-graphql/pull/1356)

# [6.0.6] 2023-09-04

- fixed SDL formatting for resolver argument comments regressed [#1363](https://github.com/async-graphql/async-graphql/issues/1363)

# [6.0.5] 2023-08-20

- Implement exporting argument documentation [#1352](https://github.com/async-graphql/async-graphql/pull/1352)
- Add `ValueAccessor::as_value` and `ListAccessor::as_values_slice` methods [#1353](https://github.com/async-graphql/async-graphql/pull/1353)
- dynamic: fixes key not found when using entity resolver [#1362](https://github.com/async-graphql/async-graphql/issues/1362)
- fix panic in complexity visitor [#1359](https://github.com/async-graphql/async-graphql/pull/1359)
- update MSRV to `1.70.0`

# [6.0.4] 2023-08-18

- Parse "repeatable" in directive definitions. [#1336](https://github.com/async-graphql/async-graphql/pull/1336)
- add support `multipart/mixed` request. [#1348](https://github.com/async-graphql/async-graphql/issues/1348)
- async-graphql-actix-web: add `GraphQL` handler.
- async-graphql-axum: add `GraphQL` service.

# [6.0.3] 2023-08-15

- dynamic: fix the error that some methods of `XXXAccessor` return reference lifetimes that are smaller than expected.
- dynamic: no longer throws an error if the Query object does not contain any fields but the schema contains entities.
- chore: make accessors public and reexport indexmap [#1329](https://github.com/async-graphql/async-graphql/pull/1329)
- feat: added `OutputType` implementation for `std::sync::Weak` [#1334](https://github.com/async-graphql/async-graphql/pull/1334)

# [6.0.1] 2023-08-02

- dynamic: remove `TypeRefInnner`
- update MSRV to `1.67.0`

# [6.0.0] 2023-07-29

- Bump `syn` from `1.0` to `2.0`
- Bump `darling` from `0.14` to `0.20`
- Bump `indexmap` from `1.6.2` to `2`
- Attributes `guard`, `process_with`, `complexity` support expression or string as value [#1295](https://github.com/async-graphql/async-graphql/issues/1295)
- Schema (type) level directive support with optional support of federation composeDirective [#1308](https://github.com/async-graphql/async-graphql/pull/1308)
- Add support for generic structs derriving InputObject and SimpleObject [#1313](https://github.com/async-graphql/async-graphql/pull/1313)
- chore: trim up some unnecessary code [#1324](https://github.com/async-graphql/async-graphql/pull/1324)
- Adds `Dataloader::get_cached_values` method to the dataloader cache so that callers can access the contents of the cache without knowing the keys. [#1326](https://github.com/async-graphql/async-graphql/pull/1326)

## Breaking Changes

- Since `syn 2.0` no longer supports keywords as meta path, rename the parameter used to specify interface field types from `type` to `ty`.

    https://github.com/dtolnay/syn/issues/1458
    https://github.com/TedDriggs/darling/issues/238

```rust
#[derive(Interface)]
#[graphql(field(name = "id", ty = "&i32"))] // rename from type to ty
enum Node {
    MyObj(MyObj),
}
```

- Change the parameter `location` of the macro `Directive` to *PascalCase*

```rust
// #[Directive(location = "field")]
#[Directive(location = "Field")]
pub fn lowercase() -> impl CustomDirective {
    LowercaseDirective
}
```

# [5.0.10] 2023-06-07

- Upgrade opentelemetry to 0.19.0 [#1252](https://github.com/async-graphql/async-graphql/pull/1262)
- Remove internal `CursorScalar` type and expose `Edge::cursor` member [#1302](https://github.com/async-graphql/async-graphql/pull/1302)

# [5.0.9] 2023-05-25

- Prevent input check stack overflow [#1293](https://github.com/async-graphql/async-graphql/pull/1293)
- Change batch requests to run concurrently [#1290](https://github.com/async-graphql/async-graphql/issues/1290)

# [5.0.8] 2023-05-09

- Improve documentation on Dataloader [#1282](https://github.com/async-graphql/async-graphql/pull/1282)
- Prevent recursive input type checking from hitting stack overflow [#1284](https://github.com/async-graphql/async-graphql/pull/1284)
- update MSRV to `1.65.0`

# [5.0.7] 2023-03-25

- Disable default-features in workspace.dependencies [#1232](https://github.com/async-graphql/async-graphql/pull/1232)
- Copy edit extensions section of The Book [#1234](https://github.com/async-graphql/async-graphql/pull/1234)
- disable default features for async-graphql in workspace dependencies [#1237](https://github.com/async-graphql/async-graphql/pull/1237)
- chore: make edge field and connection field shareable [#1246](https://github.com/async-graphql/async-graphql/pull/1246)
- Added 3 new fns to the ObjectAccessor. [#1244](https://github.com/async-graphql/async-graphql/pull/1244)
- Dataloader futures lose span context [#1256](https://github.com/async-graphql/async-graphql/pull/1256)
- Propagate ErrorExtensionValues when calling InputValueError.propagate [#1257](https://github.com/async-graphql/async-graphql/pull/1257)
- Correct error string for object in ValueAccessor [#1260](https://github.com/async-graphql/async-graphql/pull/1260)

# [5.0.6] 2023-02-11

- docs: Tweak dataloader example and link to full example [#1194](https://github.com/async-graphql/async-graphql/pull/1194)
- docs: Mention the importance of using dataloader with federation/entities [#1194](https://github.com/async-graphql/async-graphql/pull/1194)
- chore: enable GraphiQL/Playground via feature flag [#1202](https://github.com/async-graphql/async-graphql/pull/1202)
- fix: Export directives to federation SDL so they can be composed. [#1209](https://github.com/async-graphql/async-graphql/pull/1209)
- Fix doc contents details and add AutoCorrect lint to CI. [#1210](https://github.com/async-graphql/async-graphql/pull/1210)
- fix: provide correct type for _service with dynamic schema [#1212](https://github.com/async-graphql/async-graphql/pull/1212)
- feat(subscription): support generics in MergedSubscription types [#1222](https://github.com/async-graphql/async-graphql/pull/1222)
- feat: modify Connection to allow optionally disable nodes field in gql output. [#1218](https://github.com/async-graphql/async-graphql/pull/1218)
- fixes interface type condition query [#1228](https://github.com/async-graphql/async-graphql/pull/1228)
- fixes [#1226](https://github.com/async-graphql/async-graphql/issues/1226)
- update MSRV to `1.64.0`

# [5.0.5] 2023-01-03

- dynamic schema: add boxed_any function [#1179](https://github.com/async-graphql/async-graphql/pull/1179)
- Improve GraphiQL v2 [#1182](https://github.com/async-graphql/async-graphql/pull/1182)
- Fix: __Type.oneOf to __Type.isOneOf [#1188](https://github.com/async-graphql/async-graphql/pull/1188)
- Implemente From<ID> for ConstValue [#1169](https://github.com/async-graphql/async-graphql/pull/1169)
- Fixes [#1192](https://github.com/async-graphql/async-graphql/issues/1192)

# [5.0.4] 2022-12-17

- Fix named_list_nn [#1172](https://github.com/async-graphql/async-graphql/pull/1172)
- Add `DynamicRequestExt::root_value` to specify the root value for the request
- Change `CustomValidator::check` returns error type from `String` to `InputValueError<T>`.
- Add support that custom validators can set error extensions. [#1174](https://github.com/async-graphql/async-graphql/issues/1174)

# [5.0.3] 2022-12-07

- Fixes [#1163](https://github.com/async-graphql/async-graphql/issues/1163)
- Fixes [#1161](https://github.com/async-graphql/async-graphql/issues/1161)

# [5.0.2] 2022-11-30

- Fixes [#1157](https://github.com/async-graphql/async-graphql/issues/1157)

# [5.0.1] 2022-11-29

- Add boolean dynamic ValueAccessor method [#1153](https://github.com/async-graphql/async-graphql/pull/1153)

# [5.0.0] 2022-11-27

- Update MSRV to `1.60.0`
- [async-graphql-axum] bump axum from `0.5.1` to `0.6.0` [#1106](https://github.com/async-graphql/async-graphql/issues/1106)

# [5.0.0-alpha.5] 2022-11-21

- Fixes [#1138](https://github.com/async-graphql/async-graphql/issues/1138)
- Fixes [#1140](https://github.com/async-graphql/async-graphql/issues/1140)
- Add `dynamic::Scalar::validator` method to set value validator.

# [5.0.0-alpha.4] 2022-11-12

- Add support to federation(v2) for dynamic schema

# [5.0.0-alpha.3] 2022-11-12

- Simplified way to create type reference `dynamic::TypeRef`

# [5.0.0-alpha.2] 2022-11-11

- Keep object 'implements' order stable in SDL export [#1142](https://github.com/async-graphql/async-graphql/pull/1142)
- Fix regression on `ComplexObject` descriptions [#1141](https://github.com/async-graphql/async-graphql/pull/1141)

# [5.0.0-alpha.1] 2022-11-10

- Add support for dynamic schema
- Add `tempfile` feature, enabled by default

# [4.0.17] 2022-10-24

- Add support for using `Union` and `OneofObject` on the same struct [#1116](https://github.com/async-graphql/async-graphql/issues/1116)

# [4.0.16] 2022-10-20

- Add credentials to GraphiQL 2 [#1105](https://github.com/async-graphql/async-graphql/pull/1105)
- Add TypeName support for InputObject [#1110](https://github.com/async-graphql/async-graphql/pull/1110)
- Fix error message [#1058](https://github.com/async-graphql/async-graphql/pull/1058)
- Add TypeName support for Enum, Union, OneofInputObject, Subscription, MergedObject, MergedSubscription, Scalar, Interface, Directive
- Fixes [#1052](https://github.com/async-graphql/async-graphql/issues/1052)
- Implement `CustomValidator<T>` for `F: Fn(&T) -> Result<(), E: Into<String>>`
- Add `validator` attribute to `InputObject` macro [#1072](https://github.com/async-graphql/async-graphql/issues/1072)

# [4.0.15] 2022-10-07

- Dynamic Document Title for GraphiQL v2 and GraphQL Playground [#1099](https://github.com/async-graphql/async-graphql/pull/1099)
- Skip tracing for introspection queries. [#841](https://github.com/async-graphql/async-graphql/issues/841)
- Add `SchemaBuilder::disable_suggestions` method to disable field suggestions. [#1101](https://github.com/async-graphql/async-graphql/issues/1101)

# [4.0.14] 2022-09-25

- Implement a simple approach to using the link directive. [#1060](https://github.com/async-graphql/async-graphql/pull/1060)
- docs: Update federation docs with examples of each directive. [#1080](https://github.com/async-graphql/async-graphql/pull/1080)
- Add support for parse request from query string. [#1085](https://github.com/async-graphql/async-graphql/issues/1085)

# [4.0.13] 2022-09-09

- Compare to expected schema [#1048](https://github.com/async-graphql/async-graphql/pull/1048)
- docs: readme flair [#1054](https://github.com/async-graphql/async-graphql/pull/1054)
- Remove `bson-uuid` feature [#1032](https://github.com/async-graphql/async-graphql/issues/1032)
- Add `no_cache` for `cache_control` attribute [#1051](https://github.com/async-graphql/async-graphql/issues/1051)
- Resurrect code generation through tests [#1062](https://github.com/async-graphql/async-graphql/pull/1062)
- Support for primitive type in CursorType [#1049](https://github.com/async-graphql/async-graphql/pull/1049)
- Add `SDLExportOptions::include_specified_by` method to enable `specifiedBy` directive [#1065](https://github.com/async-graphql/async-graphql/issues/1065)

# [4.0.12] 2022-08-24

- Update MSRV to `1.59.0`
- Support `@specifiedBy` directive in SDL export [#1041](https://github.com/async-graphql/async-graphql/pull/1041)
- Add GraphiQL v2 [#1044](https://github.com/async-graphql/async-graphql/pull/1044)
- Export SDL: consistently avoid trailing spaces [#1043](https://github.com/async-graphql/async-graphql/pull/1043)

# [4.0.11] 2022-08-23

- Define `override` directive on fields [#1029](https://github.com/async-graphql/async-graphql/pull/1029)
- Add `@tag` support [#1038](https://github.com/async-graphql/async-graphql/pull/1038)
- Export SDL: avoid trailing space for scalar definitions [#1036](https://github.com/async-graphql/async-graphql/pull/1036)
- Fixes [#1039](https://github.com/async-graphql/async-graphql/issues/1039)

# [4.0.10] 2022-08-18

- Fixes extension `request.data(X)` being lost in the resolver [#1018](https://github.com/async-graphql/async-graphql/pull/1018)
- Add Apollo federation `@shareable` directive support [#1025](https://github.com/async-graphql/async-graphql/pull/1025)
- Add Apollo Federation `@inaccessible` directive support [#1026](https://github.com/async-graphql/async-graphql/pull/1026)

# [4.0.9] 2022-08-15

- `on_connection_init` takes `FnOnce` instead of `Fn` [#1022](https://github.com/async-graphql/async-graphql/issues/1022#issuecomment-1214575590)

# [4.0.8] 2022-08-12

- Add tracing to dataloader methods when the tracing feature is enabled. [#996](https://github.com/async-graphql/async-graphql/pull/996)

# [4.0.7] 2022-08-09

- Limit parser recursion depth to `64`.

# [4.0.6] 2022-07-21

- Limit execution recursion depth to `32` by default.

# [4.0.5] 2022-07-18

- Fix serializing of JSON default values [#969](https://github.com/async-graphql/async-graphql/issues/969)
- Bump `rocket-0.5.0-rc.1` to `rocket-0.5.0-rc.2` for `async-graphql-rocket` [#968](https://github.com/async-graphql/async-graphql/pull/968)
- Implement `Default` for `StringNumber` [#980](https://github.com/async-graphql/async-graphql/issues/980)
- Implement `Guard` for `Fn`
- Fix impossible to specify both `name` and `input_name` [#987](https://github.com/async-graphql/async-graphql/issues/987)


# [4.0.4] 2022-6-25

- Bump Actix-web from `4.0.1` to `4.1.0`
- Add a `prefer_single_line_descriptions` option on `SDLExportOptions` [#955](https://github.com/async-graphql/async-graphql/pull/955)
- Fixes [#957](https://github.com/async-graphql/async-graphql/issues/957)
- Fixes [#943](https://github.com/async-graphql/async-graphql/issues/943)

# [4.0.3] 2022-6-20

- Custom error type in axum request extractor [#945](https://github.com/async-graphql/async-graphql/pull/945)
- Add nodes exposure on `ConectionType` so nesting through edges isn't always needed. [#952](https://github.com/async-graphql/async-graphql/pull/952)
- Make email-validator optional [#950](https://github.com/async-graphql/async-graphql/pull/950)

# [4.0.2] 2022-6-10

- Expose `Edge::node` to allow better testing. [#933](https://github.com/async-graphql/async-graphql/pull/933)
- Integrate with the [`bigdecimal` crate](https://crates.io/crates/bigdecimal). [#926](https://github.com/async-graphql/async-graphql/pull/926)
- Improve the examples in the book. [#940](https://github.com/async-graphql/async-graphql/pull/940)
- Fixed [#941](https://github.com/async-graphql/async-graphql/issues/941)
- Fixed [#848](https://github.com/async-graphql/async-graphql/issues/848)
- Bump `darling` from `0.13.0` to `0.14.0` [#939](https://github.com/async-graphql/async-graphql/pull/939)
- Fixed [#9461](https://github.com/async-graphql/async-graphql/issues/946)

# [4.0.1] 2022-5-24

- Add `Schema::build_with_ignore_name_conflicts` method to specifies a list to ignore type conflict detection.

# [4.0.0] 2022-5-17

- Implement the `ConnectionNameType` and `EdgeNameType` traits to specify GraphQL type names for `Connection` and `Edge`, which can be automatically generated using `DefaultConnectionName` and `DefaultEdgeName`.
- Add `#[non_exhaustive]` attribute to Request/Response types.
- Introduce ability to pre-parse Request's query. [#891](https://github.com/async-graphql/async-graphql/pull/891)
- Add `introspection-only` mode. [#894](https://github.com/async-graphql/async-graphql/pull/894)
- Add `bson-uuid` feature to implement `ScalarType` for `bson::Uuid`. [#875](https://github.com/async-graphql/async-graphql/pull/875)
- Bump `regex` crate from `1.4.5` to `1.5.5`. [#862](https://github.com/async-graphql/async-graphql/pull/862)
- Bump `chrono-tz` crate from `0.5.3` to `0.6.1`. [#831](https://github.com/async-graphql/async-graphql/pull/831)
- Move the pest parser code generation step into a test. [#901](https://github.com/async-graphql/async-graphql/pull/901)
- Update `log` to version `0.4.16`. [#903](https://github.com/async-graphql/async-graphql/pull/903)
- Added impl of `CursorType` for floats [#897](https://github.com/async-graphql/async-graphql/pull/897)
- Implement `OutputType` for `tokio::sync::RwLock` and `tokio::sync::Mutex`. [#896](https://github.com/async-graphql/async-graphql/pull/896)
- Bump [`uuid`](https://crates.io/crates/uuid) to `1.0.0`. [#907](https://github.com/async-graphql/async-graphql/pull/907/files)
- Add some options for exporting SDL. [#877](https://github.com/async-graphql/async-graphql/issues/877)
- Cache parsed `ExecuteDocument` in APQ. [#919](https://github.com/async-graphql/async-graphql/issues/919)
- Fixed `OneofObject` restriction on inner types being unique. [#923](https://github.com/async-graphql/async-graphql/issues/923)

# [3.0.38] 2022-4-8

- Update Axum integration to Axum 0.5.1 [#883](https://github.com/async-graphql/async-graphql/pull/883)
- Support macro type in enum variant. [#884](https://github.com/async-graphql/async-graphql/pull/884)
- Introduce process_with for input object [#817](https://github.com/async-graphql/async-graphql/pull/817)
- Add `MaybeUndefined::update_to` method. [#881](https://github.com/async-graphql/async-graphql/issues/881)

# [3.0.37] 2022-3-30

- Panics when the same Rust type has the same name. [#880](https://github.com/async-graphql/async-graphql/issues/880)

# [3.0.36] 2022-3-22

- Generate `@deprecated` to SDL. [#874](https://github.com/async-graphql/async-graphql/issues/874)
- Expose `Connection::edges`. [#871](https://github.com/async-graphql/async-graphql/issues/871)

# [3.0.35] 2022-3-14

- Make `HashMap` more generics for `InputOutput` and `OutputType`.
- Add support `group` attribute to Object/SimpleObject/ComplexObject/Subscription macros. [#838](https://github.com/async-graphql/async-graphql/issues/838)
- Fixed recursive generic input objects failing to compile. [#859](https://github.com/async-graphql/async-graphql/issues/859)
- Add `ErrorExtensionValues::get` method. [#855](https://github.com/async-graphql/async-graphql/issues/855)

# [3.0.34] 2022-3-5

- Export `@oneOf` directive to SDL when Oneof type is defined. [#766](https://github.com/async-graphql/async-graphql/issues/766)

# [3.0.33] 2022-3-4

- Add support for oneof field on object. [#766](https://github.com/async-graphql/async-graphql/issues/766)

# [3.0.32] 2022-3-4

- Bump `Actix-web` from `4.0.0-rc.3` to `4.0.1`.

# [3.0.31] 2022-02-17

- Add `OneOfObject` macro to support for oneof input object.
- Bump actix-web from `4.0.0-rc.2` to `4.0.0-rc.3`.

# [3.0.30] 2022-2-15

- Implement `ScalarType` for `time::Date`. [#822](https://github.com/async-graphql/async-graphql/pull/822)

# [3.0.29] 2022-2-6

- Pass context to resolvers with flatten attribute. [#813](https://github.com/async-graphql/async-graphql/pull/813)
- Add support for using both `ComplexObject` and `InputObject`.
- Bump `Actix-web` from `4.0.0-beta.19` to `4.0.0-rc.2`.

# [3.0.28] 2022-1-30

- Implement `InputType` and `OutputType` for `Box<[T]>` and `Arc<[T]>`. [#805](https://github.com/async-graphql/async-graphql/issues/805)

# [3.0.27] 2022-1-28

- Fix possible stack overflow in validator, thanks @quapka.

# [3.0.26] 2022-1-26

- Add `skip_input` attribute to `InputObject` macro, `skip_output` attribute to `SimpleObject` macro.

# [3.0.25] 2022-1-24

- Fixed some integrations overwritten HTTP headers. [#793](https://github.com/async-graphql/async-graphql/issues/793)
- Fixed variable type not checked when given a default value. [#795](https://github.com/async-graphql/async-graphql/pull/795)

# [3.0.24] 2022-1-24

- Remove `'static` bound for `impl From<T> for Error`.

# [3.0.23] 2022-1-19

- Bump hashbrown from `0.11.2` to `0.12.0`.
- Implement `InputType` for `Box<str>` and `Arc<str>`. [#792](https://github.com/async-graphql/async-graphql/issues/792)
- Add scalars for the `time` crate's datetime types. [#791](https://github.com/async-graphql/async-graphql/pull/791)
- Add `DataContext` trait. [#786](https://github.com/async-graphql/async-graphql/pull/786)

## [3.0.22] 2022-1-11

- Add support `flatten` attribute for `SimpleObject`, `ComplexObject` and `Object` macros. [#533](https://github.com/async-graphql/async-graphql/issues/533)
- Actix integration: cbor response support + error handling improvements [#784](https://github.com/async-graphql/async-graphql/pull/784)

## [3.0.21] 2022-1-11

- Add `Union` and `Interface` support for trait objects. [#780](https://github.com/async-graphql/async-graphql/issues/780)

## [3.0.20] 2022-1-5

- Bump `lru` to `0.7.1`. [#773](https://github.com/async-graphql/async-graphql/pull/773)
- Align `indexmap` version to `1.6.2`. [#776](https://github.com/async-graphql/async-graphql/pull/776)
- Bump actix-web from `4.0.0-beta.18` to `4.0.0-beta.19`.
- Fix the generic `SimpleObject` can't define the lifetimes. [#774](https://github.com/async-graphql/async-graphql/issues/774)

## [3.0.19] 2021-12-28

- Add `InputType` / `OutputType` support for `hashbrown` crate.
- Bump actix-web from `4.0.0-beta.14` to `4.0.0-beta.18`. [#768](https://github.com/async-graphql/async-graphql/pull/768)

## [3.0.18] 2021-12-26

- Federation's `_Entity` should not be sent if empty as it's in conflict with [GraphQL Union type validation](https://spec.graphql.org/draft/#sec-Unions.Type-Validation) [#765](https://github.com/async-graphql/async-graphql/pull/765).
- Fix field guards not working on `ComplexObject`. [#767](https://github.com/async-graphql/async-graphql/issues/767)

## [3.0.17] 2021-12-16

- Bump poem to `1.2.2`.

## [3.0.16] 2021-12-16

- Bump poem to `1.2.1`.

## [3.0.15] 2021-12-12

- Bump actix-web from `4.0.0-beta.11` to `4.0.0-beta.14`.

## [3.0.14] 2021-12-06

- [async-graphql-axum] bump axum from `0.3` to `0.4`.

## [3.0.13] 2021-12-06

- No longer assumes that a subscription stream that failed to resolve has ended. [#744](https://github.com/async-graphql/async-graphql/issues/744)
- Rework to implement `InputType` and `OutputType` for `HashMap` and `BTreeMap`.

## [3.0.12] 2021-12-05

- Fix possible deadlock in dataloader. [#555](https://github.com/async-graphql/async-graphql/issues/555)
- Add some helper methods for `BatchRequest`.
  - BatchRequest::iter
  - BatchRequest::iter_mut
  - BatchRequest::variables
  - BatchRequest::data
  - BatchRequest::disable_introspection
- Fix implicit interfaces not being exposed via the __schema introspection. [#741](https://github.com/async-graphql/async-graphql/pull/741)

## [3.0.11] 2021-12-02

- Fix panic on f32-64::INFINITE/f32-64::NEG_INFINITE/f32-64::NAN output. [#735](https://github.com/async-graphql/async-graphql/issues/735)

## [3.0.10] 2021-11-30

- Fix the custom validator cannot work on `Option<Vec<T>>`.

## [3.0.9] 2021-11-30

- Fix the validator cannot work on `Option<Vec<T>>`.

## [3.0.8] 2021-11-30

- `#[graphql(validator(list))]` no longer applies to `max_items` and `min_items`.
- Implement `InputValue`/`OutputValue` for `serde_json::Value`.
- Add support for `SmolStr` via a feature. [#730](https://github.com/async-graphql/async-graphql/pull/730)

## [3.0.7] 2021-11-23

- Fix error extensions cause stack overflow. [#719](https://github.com/async-graphql/async-graphql/issues/719)

## [3.0.6] 2021-11-19

- Custom directives. [Book](https://async-graphql.github.io/async-graphql/en/custom_directive.html)

## [3.0.5] 2021-11-19

- Remove skipped fields from the document before executing the query.
- Add `isRepeatable` field to `__Directive` - [GraphQL - October 2021]

## [3.0.4] 2021-11-18

- Remove `OutputJson` because `Json` can replace it.
- Allowed use validators on wrapper types, for example: `Option<T>`, `MaybeUnefined<T>`.

## [3.0.3] 2021-11-18

- [integrations] Make `GraphQLWebSocket::new` use generic stream.
- [integrations] Add `GraphQLWebSocket::new_with_pair` method.

## [3.0.2] 2021-11-16

- Add `url`, `regex` and `ip` validators.

## [3.0.1] 2021-11-17

- Remove the `ctx` parameter of `CustomValidator::check`. [#710](https://github.com/async-graphql/async-graphql/issues/710)

## [3.0.0-alpha.2] 2021-11-16

- Change the signature of the `connection::query` function to allow the callback to use any type that implements `Into<Error>`.
- Remove `ResolverError` and use `Error::new_with_source` instead.
- Add `ErrorExtensionValues::unset` method.
- Use the `SimpleObject` macro and the `InputObject` macro at the same time.
- Types that are not referenced will be hidden in introspection.
- Make the API of integrations is more consistent.
- Remove `async-graphql-tide`.
- Rework validators. [Book](https://async-graphql.github.io/async-graphql/en/input_value_validators.html)
- Rework guards. [Book](https://async-graphql.github.io/async-graphql/en/field_guard.html)

## [2.11.3] 2021-11-13

- Implemented CursorType for i32/i64. [#701](https://github.com/async-graphql/async-graphql/pull/701)
- An error is returned when the number fails to parse. [#704](https://github.com/async-graphql/async-graphql/issues/704)
- Fix Federation entity union is empty during schema introspection. [#700](https://github.com/async-graphql/async-graphql/issues/700)

## [2.11.2] 2021-11-11

- Fix the problem that `EmptyMutation` may cause when used in `MergedObject`. [#694](https://github.com/async-graphql/async-graphql/issues/694)
- If a GraphQL name conflict is detected when creating schema, it will cause panic. [#499](https://github.com/async-graphql/async-graphql/issues/499)

## [2.11.1] 2021-11-07

- Add `chrono::Duration` custom scalar. [#689](https://github.com/async-graphql/async-graphql/pull/689)
- Implement `From<Option<Option<T>>>` for `MaybeUndefined<T>`.
- Add `MaybeUndefined::as_opt_ref`, `MaybeUndefined::as_opt_deref`, `MaybeUndefined::map`, `MaybeUndefined::map_value`, `MaybeUndefined::contains`, `MaybeUndefined::contains_value`, and `MaybeUndefined::transpose` methods.
- Made `MaybeUndefined::is_undefined`, `MaybeUndefined::is_null`, `MaybeUndefined::is_value`, `MaybeUndefined::value` and `MaybeUndefined::as_opt_ref` const.
- Add `ResolverError` type. [#671](https://github.com/async-graphql/async-graphql/issues/671)
- [async-graphql-axum] Bump axum from `0.2.5` to `0.3`.
- [async-graphql-poem] Export the HTTP headers in the `Context`.

## [2.11.0] 2021-11-03

- Use Rust `2021` edition.
- Subscription typename - [GraphQL - October 2021] [#681](https://github.com/async-graphql/async-graphql/issues/681)
- Allow directive on variable definition - [GraphQL - October 2021] [#678](https://github.com/async-graphql/async-graphql/issues/678)
- Specified By - [GraphQL - October 2021] [#677](https://github.com/async-graphql/async-graphql/issues/677)
- Add `specified_by_url` for `Tz`, `DateTime<Tz>`, `Url`, `Uuid` and `Upload` scalars.
- Number value literal lookahead restrictions - [GraphQL - October 2021] [#685](https://github.com/async-graphql/async-graphql/issues/685)

## [2.10.8] 2021-10-26

- [async-graphql-poem] Bump poem to `1.0.13`. 

## [2.10.6] 2021-10-26

- Add derived for object & simple object & complex object. [#667](https://github.com/async-graphql/async-graphql/pull/667) [#670](https://github.com/async-graphql/async-graphql/pull/670)
- Respect query object field order. [#612](https://github.com/async-graphql/async-graphql/issues/612)

## [2.10.5] 2021-10-22

- Bump poem from `0.6.6` to `1.0.7`. 

## [2.10.4] 2021-10-22

- Implement `Default` for ID #659.
- Implement `ScalarType` for `bson::Bson` and `bson::Document`. [#661](https://github.com/async-graphql/async-graphql/pull/661)
- Add `CharsMinLength` and `CharsMaxLength` validators. [#656](https://github.com/async-graphql/async-graphql/pull/656)
- Fix the `Subscription` macro to work on Rust 2021. [#665](https://github.com/async-graphql/async-graphql/pull/665)

## [2.10.3] 2021-10-12

- Add `visible` macro argument for union type. [#655](https://github.com/async-graphql/async-graphql/pull/655)

## [2.10.2] 2021-09-29

- Add concrete names support for `Object` macro. [#633](https://github.com/async-graphql/async-graphql/issues/633)
- Add `Lookahead::selection_fields` method. [#643](https://github.com/async-graphql/async-graphql/pull/643)

## [2.10.1] 2021-09-24

- Add `DataLoader::enable_all_cache` and `DataLoader::enable_cache` methods. [#642](https://github.com/async-graphql/async-graphql/issues/642)
- Change the execution order of `chain` and `race` guards. [#614](https://github.com/async-graphql/async-graphql/issues/614)
- Change log level from `error` to `info`. [#518](https://github.com/async-graphql/async-graphql/issues/518)

## [2.10.0] 2021-09-17

- Add support for `graphql-ws` pings. [#635](https://github.com/async-graphql/async-graphql/issues/635)
- Add feature gate `websocket` for async-graphql-tide. [#636](https://github.com/async-graphql/async-graphql/issues/636)
- Implement GraphQL enum to `Value` conversion. [#617](https://github.com/async-graphql/async-graphql/issues/617)
- Implement `ScalarType` for `HashMap`/`BTreeMap` to use `ToString`/`FromStr`. [#585](https://github.com/async-graphql/async-graphql/issues/585)

## [2.9.15] 2021-09-10

- Added Axum error handling. [#629](https://github.com/async-graphql/async-graphql/pull/629)
- Bump bson from `2.0.0-beta.1` to `2.0.0`. [#628](https://github.com/async-graphql/async-graphql/pull/628)

## [2.9.14] 2021-09-03

- Add support for [serde_cbor](https://crates.io/crates/serde_cbor). [#619](https://github.com/async-graphql/async-graphql/pull/619)

## [2.9.13] 2021-09-01

- Released [`Axum`](https://github.com/tokio-rs/axum) integration. [`async-graphql-axum`](https://crates.io/crates/async-graphql-axum)

## [2.9.12] 2021-08-24

- Add integration for [`Poem`](https://github.com/poem-web/poem).
- Ignore items flagged `@skip` in `SelectionField` and `Lookahead`. [#605](https://github.com/async-graphql/async-graphql/pull/605)

## [2.9.11] 2021-08-22

- Implement `From<MaybeUndefined<T>> for Option<Option<T>>`. [#599](https://github.com/async-graphql/async-graphql/issues/599)
- Add human readable for serializer. [#604](https://github.com/async-graphql/async-graphql/pull/604)

## [2.9.10] 2021-08-05

- Change `GraphQLPlaygroundConfig::with_setting` to accept `impl Into<Value>` [#583](https://github.com/async-graphql/async-graphql/issues/583)
- Remove unnecessary unwrap in multipart handler [#594](https://github.com/async-graphql/async-graphql/pull/594)

## [2.9.9] 2021-07-20

- Add binary types to `ConstValue` and `Value`. [#569](https://github.com/async-graphql/async-graphql/issues/569)
- Implemented `OutputType` for [Bytes](https://docs.rs/bytes/1.0.1/bytes/struct.Bytes.html).
- Changed Lookahead to support multiple fields. [#574](https://github.com/async-graphql/async-graphql/issues/574)
- Implement `TryFrom<&[SelectionField<'a>]>` for `Lookahead<'a>`. [#575](https://github.com/async-graphql/async-graphql/issues/575)
- Attach custom HTTP headers to the response when an error occurs. [#572](https://github.com/async-graphql/async-graphql/issues/572)
- Allow field visible to support paths. [#578](https://github.com/async-graphql/async-graphql/pull/578)
- Add `list` attribute to the input value validator. [#579](https://github.com/async-graphql/async-graphql/issues/579)

## [2.9.8] 2021-07-12

- Add Extensions in Error of `InputValueValidator`. [#564](https://github.com/async-graphql/async-graphql/pull/564)

- Fix SDL print is not stable. [#547](https://github.com/async-graphql/async-graphql/issues/547)

## [2.9.7] 2021-07-04

- Add support for generic `ComplexObject`. [#562](https://github.com/async-graphql/async-graphql/pull/562)

## [2.9.6] 2021-07-02

- Implement `From<SelectionField>` for `Lookahead`. [#557](https://github.com/async-graphql/async-graphql/issues/557)
  
- Add Decimal scalar (from `rust_decimal` crate) [#559](https://github.com/async-graphql/async-graphql/pull/559)

## [2.9.5] 2021-06-29

- Allows to get the actual field name and alias in `ResolveInfo`. [#551](https://github.com/async-graphql/async-graphql/issues/551)

## [2.9.4] 2021-06-21

- Fix the bug that `MergedObject` may cause panic. [#539](https://github.com/async-graphql/async-graphql/issues/539#issuecomment-862209442)

## [2.9.3] 2021-06-17

- Bump upstream crate `bson` from `v1.2.0` to `v2.0.0-beta.1`. [#516](https://github.com/async-graphql/async-graphql/pull/516)

- Add `serial` attribute for `Object`, `SimpleObject` and `MergedObject` macros. [#539](https://github.com/async-graphql/async-graphql/issues/539)

- Remove the `static` constraint of the `receive_body` and `receive_batch_body` functions. [#544](https://github.com/async-graphql/async-graphql/issues/544)

- Implement `InputType` and `OutputType` for `[T; N]` array.

## [2.9.2] 2021-06-10

- Allow field guards to support paths. [#536](https://github.com/async-graphql/async-graphql/issues/536)
  
- Add the `operation_name` to `Extension::execute` method. [#538](https://github.com/async-graphql/async-graphql/issues/538)

## [2.9.1] 2021-06-08

- Rework error propagation. [#531](https://github.com/async-graphql/async-graphql/issues/531)

## [2.9.0] 2021-06-07

- Add support for returning multiple resolver errors. [#531](https://github.com/async-graphql/async-graphql/issues/531)

- Bump upstream crate `multer` from `v1.2.2` to `v2.0.0`.

- Aligned NaiveDateTime formatting with DateTime. [#535](https://github.com/async-graphql/async-graphql/pull/535)

## [2.8.6] 2021-06-01

- Allow the ability to set GraphQL Playground settings. [#508](https://github.com/async-graphql/async-graphql/pull/508)

- WebSocket is now generic in graphql_subscription_upgrade functions. [#530](https://github.com/async-graphql/async-graphql/pull/530)

- Removed `Copy` trait from initializer in `graphql_subscription_with_data`. [#530](https://github.com/async-graphql/async-graphql/pull/530)

## [2.8.5] 2021-05-11

- If `InputObject` contains an unnamed field, the correct error message will be given. [#498](https://github.com/async-graphql/async-graphql/issues/498)

- Added `Websocket::with_message_stream` for client message customization. [#501](https://github.com/async-graphql/async-graphql/pull/501)

- Added the `Secret` type using [secrecy](https://crates.io/crates/secrecy) crate.

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
