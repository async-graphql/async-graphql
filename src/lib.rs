//! # A GraphQL server library implemented in Rust
//!
//! <div align="center">
//! <!-- CI -->
//! <img src="https://github.com/async-graphql/async-graphql/workflows/CI/badge.svg" />
//! <!-- codecov -->
//! <img src="https://codecov.io/gh/async-graphql/async-graphql/branch/master/graph/badge.svg" />
//! <!-- Crates version -->
//! <a href="https://crates.io/crates/async-graphql">
//! <img src="https://img.shields.io/crates/v/async-graphql.svg?style=flat-square"
//! alt="Crates.io version" />
//! </a>
//! <!-- Downloads -->
//! <a href="https://crates.io/crates/async-graphql">
//! <img src="https://img.shields.io/crates/d/async-graphql.svg?style=flat-square"
//! alt="Download" />
//! </a>
//! <!-- docs.rs docs -->
//! <a href="https://docs.rs/async-graphql">
//! <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
//! alt="docs.rs docs" />
//! </a>
//! <a href="https://github.com/rust-secure-code/safety-dance/">
//! <img src="https://img.shields.io/badge/unsafe-forbidden-success.svg?style=flat-square"
//! alt="Unsafe Rust forbidden" />
//! </a>
//! </div>
//!
//! ## Documentation
//!
//! * [Feature Comparison](https://github.com/async-graphql/async-graphql/blob/master/feature-comparison.md)
//! * [Book](https://async-graphql.github.io/async-graphql/en/index.html)
//! * [中文文档](https://async-graphql.github.io/async-graphql/zh-CN/index.html)
//! * [Docs](https://docs.rs/async-graphql)
//! * [GitHub repository](https://github.com/async-graphql/async-graphql)
//! * [Cargo package](https://crates.io/crates/async-graphql)
//! * Minimum supported Rust version: 1.56.1 or later
//!
//! ## Features
//!
//! * Fully supports async/await
//! * Type safety
//! * Rustfmt friendly (Procedural Macro)
//! * Custom scalars
//! * Minimal overhead
//! * Easy integration ([poem](https://crates.io/crates/poem), actix_web, tide, warp, rocket ...)
//! * File upload (Multipart request)
//! * Subscriptions (WebSocket transport)
//! * Custom extensions
//! * Apollo Tracing extension
//! * Limit query complexity/depth
//! * Error Extensions
//! * Apollo Federation
//! * Batch Queries
//! * Apollo Persisted Queries
//!
//! ## Crate features
//!
//! This crate offers the following features, all of which are not activated by default:
//!
//! - `apollo_tracing`: Enable the [Apollo tracing extension](extensions/struct.ApolloTracing.html).
//! - `apollo_persisted_queries`: Enable the [Apollo persisted queries extension](extensions/apollo_persisted_queries/struct.ApolloPersistedQueries.html).
//! - `log`: Enable the [logger extension](extensions/struct.Logger.html).
//! - `tracing`: Enable the [tracing extension](extensions/struct.Tracing.html).
//! - `opentelemetry`: Enable the [OpenTelemetry extension](extensions/struct.OpenTelemetry.html).
//! - `unblock`: Support [asynchronous reader for Upload](types/struct.Upload.html)
//! - `bson`: Integrate with the [`bson` crate](https://crates.io/crates/bson).
//! - `chrono`: Integrate with the [`chrono` crate](https://crates.io/crates/chrono).
//! - `chrono-tz`: Integrate with the [`chrono-tz` crate](https://crates.io/crates/chrono-tz).
//! - `url`: Integrate with the [`url` crate](https://crates.io/crates/url).
//! - `uuid`: Integrate with the [`uuid` crate](https://crates.io/crates/uuid).
//! - `string_number`: Enable the [StringNumber](types/struct.StringNumber.html).
//! - `dataloader`: Support [DataLoader](dataloader/struct.DataLoader.html).
//! - `decimal`: Integrate with the [`rust_decimal` crate](https://crates.io/crates/rust_decimal).
//! - `cbor`: Support for [serde_cbor](https://crates.io/crates/serde_cbor).
//!
//! ## Integrations
//!
//! * Actix-web [async-graphql-actix_web](https://crates.io/crates/async-graphql-actix-web)
//! * Warp [async-graphql-warp](https://crates.io/crates/async-graphql-warp)
//! * Tide [async-graphql-tide](https://crates.io/crates/async-graphql-tide)
//!
//! ## License
//!
//! Licensed under either of
//!
//! * Apache License, Version 2.0,
//! (./LICENSE-APACHE or <http://www.apache.org/licenses/LICENSE-2.0>)
//! * MIT license (./LICENSE-MIT or <http://opensource.org/licenses/MIT>)
//! at your option.
//!
//! ## References
//!
//! * [GraphQL](https://graphql.org)
//! * [GraphQL Multipart Request](https://github.com/jaydenseric/graphql-multipart-request-spec)
//! * [GraphQL Cursor Connections Specification](https://facebook.github.io/relay/graphql/connections.htm)
//! * [GraphQL over WebSocket Protocol](https://github.com/apollographql/subscriptions-transport-ws/blob/master/PROTOCOL.md)
//! * [Apollo Tracing](https://github.com/apollographql/apollo-tracing)
//! * [Apollo Federation](https://www.apollographql.com/docs/apollo-server/federation/introduction)
//!
//! ## Examples
//!
//! All examples are in the [sub-repository](https://github.com/async-graphql/examples), located in the examples directory.
//!
//! **Run an example:**
//!
//! ```shell
//! git submodule update # update the examples repo
//! cd examples && cargo run --bin [name]
//! ```
//!
//! ## Benchmarks
//!
//! Ensure that there is no CPU-heavy process in background!
//!
//! ```shell script
//! cd benchmark
//! cargo bench
//! ```
//!
//! Now a HTML report is available at `benchmark/target/criterion/report`.
//!

#![deny(clippy::all)]
// #![deny(clippy::pedantic)]
#![deny(clippy::inefficient_to_string)]
#![deny(clippy::match_wildcard_for_single_variants)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::similar_names)]
#![allow(clippy::if_not_else)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::needless_pass_by_value)]
#![deny(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::map_flatten)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::explicit_iter_loop)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::unused_self)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::implicit_hasher)]
// #![deny(clippy::nursery)]
#![allow(clippy::use_self)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::future_not_send)]
#![allow(clippy::redundant_pub_crate)]
#![allow(clippy::cognitive_complexity)]
#![allow(clippy::useless_let_if_seq)]
#![warn(missing_docs)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::upper_case_acronyms)]
#![recursion_limit = "256"]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod base;
mod error;
mod look_ahead;
mod model;
mod request;
mod response;
mod schema;
mod subscription;
mod validation;

pub mod context;
#[cfg(feature = "dataloader")]
#[cfg_attr(docsrs, doc(cfg(feature = "dataloader")))]
pub mod dataloader;
pub mod extensions;
pub mod guard;
pub mod http;
pub mod resolver_utils;
pub mod types;
pub mod validators;

#[doc(hidden)]
pub mod registry;

#[doc(hidden)]
pub use async_stream;
#[doc(hidden)]
pub use async_trait;
#[doc(hidden)]
pub use context::ContextSelectionSet;
#[doc(hidden)]
pub use futures_util;
#[doc(hidden)]
pub use indexmap;
#[doc(hidden)]
pub use static_assertions;
#[doc(hidden)]
pub use subscription::SubscriptionType;

pub use async_graphql_parser as parser;
pub use async_graphql_value::{
    from_value, to_value, value, ConstValue as Value, DeserializerError, Name, Number,
    SerializerError, Variables,
};
pub use base::{
    ComplexObject, Description, InputObjectType, InputType, InterfaceType, ObjectType, OutputType,
    UnionType,
};
pub use error::{
    Error, ErrorExtensionValues, ErrorExtensions, InputValueError, InputValueResult,
    ParseRequestError, PathSegment, ResolverError, Result, ResultExt, ServerError, ServerResult,
};
pub use look_ahead::Lookahead;
pub use registry::CacheControl;
pub use request::{BatchRequest, Request};
#[doc(no_inline)]
pub use resolver_utils::{ContainerType, EnumType, ScalarType};
pub use response::{BatchResponse, Response};
pub use schema::{Schema, SchemaBuilder, SchemaEnv};
pub use validation::{ValidationMode, ValidationResult, VisitorContext};

pub use context::*;
#[doc(no_inline)]
pub use parser::{Pos, Positioned};
pub use types::*;

/// An alias of [async_graphql::Error](struct.Error.html). Present for backward compatibility
/// reasons.
pub type FieldError = Error;

/// An alias of [async_graphql::Result](type.Result.html). Present for backward compatibility
/// reasons.
pub type FieldResult<T> = Result<T>;

/// Define a GraphQL object with methods
///
/// *[See also the Book](https://async-graphql.github.io/async-graphql/en/define_complex_object.html).*
///
/// All methods are converted to camelCase.
///
/// # Macro parameters
///
/// | Attribute     | description               | Type     | Optional |
/// |---------------|---------------------------|----------|----------|
/// | name          | Object name               | string   | Y        |
/// | rename_fields | Rename all the fields according to the given case convention. The possible values are "lowercase", "UPPERCASE", "PascalCase", "camelCase", "snake_case", "SCREAMING_SNAKE_CASE".| string   | Y        |
/// | rename_args   | Rename all the arguments according to the given case convention. The possible values are "lowercase", "UPPERCASE", "PascalCase", "camelCase", "snake_case", "SCREAMING_SNAKE_CASE".| string   | Y        |
/// | cache_control | Object cache control      | [`CacheControl`](struct.CacheControl.html) | Y        |
/// | extends       | Add fields to an entity that's defined in another service | bool | Y |
/// | use_type_description | Specifies that the description of the type is on the type declaration. [`Description`]()(derive.Description.html) | bool | Y |
/// | visible       | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool | Y |
/// | visible       | Call the specified function. If the return value is `false`, it will not be displayed in introspection. | string | Y |
///
/// # Field parameters
///
/// | Attribute     | description               | Type     | Optional |
/// |---------------|---------------------------|----------|----------|
/// | skip          | Skip this field           | bool     | Y        |
/// | name          | Field name                | string   | Y        |
/// | desc          | Field description         | string   | Y        |
/// | deprecation   | Field deprecated          | bool     | Y        |
/// | deprecation   | Field deprecation reason  | string   | Y        |
/// | cache_control | Field cache control       | [`CacheControl`](struct.CacheControl.html) | Y        |
/// | external      | Mark a field as owned by another service. This allows service A to use fields from service B while also knowing at runtime the types of that field. | bool | Y |
/// | provides      | Annotate the expected returned fieldset from a field on a base type that is guaranteed to be selectable by the gateway. | string | Y |
/// | requires      | Annotate the required input fieldset from a base type for a resolver. It is used to develop a query plan where the required fields may not be needed by the client, but the service may need additional information from other services. | string | Y |
/// | guard         | Field of guard            | [`Guard`](guard/trait.Guard.html) | Y        |
/// | visible       | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool | Y |
/// | visible       | Call the specified function. If the return value is `false`, it will not be displayed in introspection. | string | Y |
///
/// # Field argument parameters
///
/// | Attribute    | description                              | Type        | Optional |
/// |--------------|------------------------------------------|------------ |----------|
/// | name         | Argument name                            | string      | Y        |
/// | desc         | Argument description                     | string      | Y        |
/// | default      | Use `Default::default` for default value | none        | Y        |
/// | default      | Argument default value                   | literal     | Y        |
/// | default_with | Expression to generate default value     | code string | Y        |
/// | derived      | Generate derived fields *[See also the Book](https://async-graphql.github.io/async-graphql/en/derived_fields.html).*                 | object        | Y        |
/// | validator    | Input value validator                    | [`InputValueValidator`](validators/trait.InputValueValidator.html) | Y        |
/// | complexity   | Custom field complexity. *[See also the Book](https://async-graphql.github.io/async-graphql/en/depth_and_complexity.html).*                 | bool        | Y        |
/// | complexity   | Custom field complexity.                 | string      | Y        |
/// | visible      | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool | Y |
/// | visible      | Call the specified function. If the return value is `false`, it will not be displayed in introspection. | string | Y |
/// | secret       | Mark this field as a secret, it will not output the actual value in the log. | bool | Y |
/// | serial       | Resolve each field sequentially.         | bool        | Y        |
/// | key          | Is entity key(for Federation)            | bool        | Y        |
///
/// # Derived argument parameters
///
/// | Attribute    | description                              | Type        | Optional |
/// |--------------|------------------------------------------|------------ |----------|
/// | name         | Generated derived field name             | string      | N        |
/// | into         | Type to derived an into                  | string      | Y        |
/// | with         | Function to apply to manage advanced use cases | string| Y        |
///
/// # Valid field return types
///
/// - Scalar values, such as `i32` and `bool`. `usize`, `isize`, `u128` and `i128` are not
/// supported
/// - `Vec<T>`, such as `Vec<i32>`
/// - Slices, such as `&[i32]`
/// - `Option<T>`, such as `Option<i32>`
/// - `BTree<T>`, `HashMap<T>`, `HashSet<T>`, `BTreeSet<T>`, `LinkedList<T>`, `VecDeque<T>`
/// - GraphQL objects.
/// - GraphQL enums.
/// - References to any of the above types, such as `&i32` or `&Option<String>`.
/// - `Result<T, E>`, such as `Result<i32, E>`
///
/// # Context
///
/// You can define a context as an argument to a method, and the context should be the first argument to the method.
///
/// ```ignore
/// #[Object]
/// impl QueryRoot {
///     async fn value(&self, ctx: &Context<'_>) -> { ... }
/// }
/// ```
///
/// # Examples
///
/// Implements GraphQL Object for struct.
///
/// ```rust
/// use async_graphql::*;
///
/// struct QueryRoot {
///     value: i32,
/// }
///
/// #[Object]
/// impl QueryRoot {
///     /// value
///     async fn value(&self) -> i32 {
///         self.value
///     }
///
///     /// reference value
///     async fn value_ref(&self) -> &i32 {
///         &self.value
///     }
///
///     /// value with error
///     async fn value_with_error(&self) -> Result<i32> {
///         Ok(self.value)
///     }
///
///     async fn value_with_arg(&self, #[graphql(default = 1)] a: i32) -> i32 {
///         a
///     }
/// }
///
/// tokio::runtime::Runtime::new().unwrap().block_on(async move {
///     let schema = Schema::new(QueryRoot { value: 10 }, EmptyMutation, EmptySubscription);
///     let res = schema.execute(r#"{
///         value
///         valueRef
///         valueWithError
///         valueWithArg1: valueWithArg
///         valueWithArg2: valueWithArg(a: 99)
///     }"#).await.into_result().unwrap().data;
///     assert_eq!(res, value!({
///         "value": 10,
///         "valueRef": 10,
///         "valueWithError": 10,
///         "valueWithArg1": 1,
///         "valueWithArg2": 99
///     }));
/// });
/// ```
///
/// # Examples
///
/// Implements GraphQL Object for trait object.
///
/// ```rust
/// use async_graphql::*;
///
/// trait MyTrait: Send + Sync {
///     fn name(&self) -> &str;
/// }
///
/// #[Object]
/// impl dyn MyTrait {
///     #[graphql(name = "name")]
///     async fn gql_name(&self) -> &str {
///         self.name()
///     }
/// }
///
/// struct MyObj(String);
///
/// impl MyTrait for MyObj {
///     fn name(&self) -> &str {
///         &self.0
///     }
/// }
///
/// struct QueryRoot;
///
/// #[Object]
/// impl QueryRoot {
///     async fn objs(&self) -> Vec<Box<dyn MyTrait>> {
///         vec![
///             Box::new(MyObj("a".to_string())),
///             Box::new(MyObj("b".to_string())),
///         ]
///     }
/// }
///
/// tokio::runtime::Runtime::new().unwrap().block_on(async move {
///     let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
///     let res = schema.execute("{ objs { name } }").await.into_result().unwrap().data;
///     assert_eq!(res, value!({
///         "objs": [
///             { "name": "a" },
///             { "name": "b" },
///         ]
///     }));
/// });
/// ```
pub use async_graphql_derive::Object;

/// Define a GraphQL object with fields
///
/// *[See also the Book](https://async-graphql.github.io/async-graphql/en/define_simple_object.html).*
///
/// Similar to `Object`, but defined on a structure that automatically generates getters for all fields. For a list of valid field types, see [`Object`](attr.Object.html). All fields are converted to camelCase.
///
/// # Macro parameters
///
/// | Attribute     | description               | Type     | Optional |
/// |---------------|---------------------------|----------|----------|
/// | name          | Object name               | string   | Y        |
/// | rename_fields | Rename all the fields according to the given case convention. The possible values are "lowercase", "UPPERCASE", "PascalCase", "camelCase", "snake_case", "SCREAMING_SNAKE_CASE".| string   | Y        |
/// | cache_control | Object cache control      | [`CacheControl`](struct.CacheControl.html) | Y        |
/// | extends       | Add fields to an entity that's defined in another service | bool | Y |
/// | visible       | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool | Y |
/// | visible       | Call the specified function. If the return value is `false`, it will not be displayed in introspection. | string | Y |
/// | concretes     | Specify how the concrete type of the generic SimpleObject should be implemented. *[See also the Book](https://async-graphql.github.io/async-graphql/en/define_simple_object.html#generic-simpleobjects) | ConcreteType |  Y |
/// | serial        | Resolve each field sequentially.         | bool        | Y        |
///
/// # Field parameters
///
/// | Attribute     | description               | Type     | Optional |
/// |---------------|---------------------------|----------|----------|
/// | skip          | Skip this field           | bool     | Y        |
/// | name          | Field name                | string   | Y        |
/// | deprecation   | Field deprecated          | bool     | Y        |
/// | deprecation   | Field deprecation reason  | string   | Y        |
/// | derived      | Generate derived fields *[See also the Book](https://async-graphql.github.io/async-graphql/en/derived_fields.html).*                 | object        | Y        |
/// | owned         | Field resolver return a ownedship value  | bool   | Y        |
/// | cache_control | Field cache control       | [`CacheControl`](struct.CacheControl.html) | Y        |
/// | external      | Mark a field as owned by another service. This allows service A to use fields from service B while also knowing at runtime the types of that field. | bool | Y |
/// | provides      | Annotate the expected returned fieldset from a field on a base type that is guaranteed to be selectable by the gateway. | string | Y |
/// | requires      | Annotate the required input fieldset from a base type for a resolver. It is used to develop a query plan where the required fields may not be needed by the client, but the service may need additional information from other services. | string | Y |
/// | guard         | Field of guard            | [`Guard`](guard/trait.Guard.html) | Y        |
/// | visible       | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool | Y |
/// | visible       | Call the specified function. If the return value is `false`, it will not be displayed in introspection. | string | Y |
///
/// # Derived argument parameters
///
/// | Attribute    | description                              | Type        | Optional |
/// |--------------|------------------------------------------|------------ |----------|
/// | name         | Generated derived field name             | string      | N        |
/// | into         | Type to derived an into                  | string      | Y        |
/// | owned        | Field resolver return a ownedship value  | bool        | Y        |
/// | with         | Function to apply to manage advanced use cases | string| Y        |
///
///
/// # Examples
///
/// ```rust
/// use async_graphql::*;
///
/// #[derive(SimpleObject)]
/// struct QueryRoot {
///     value: i32,
/// }
///
/// tokio::runtime::Runtime::new().unwrap().block_on(async move {
///     let schema = Schema::new(QueryRoot{ value: 10 }, EmptyMutation, EmptySubscription);
///     let res = schema.execute("{ value }").await.into_result().unwrap().data;
///     assert_eq!(res, value!({
///         "value": 10,
///     }));
/// });
/// ```
pub use async_graphql_derive::SimpleObject;

/// Define a complex GraphQL object for SimpleObject's complex field resolver.
///
/// *[See also the Book](https://async-graphql.github.io/async-graphql/en/define_simple_object.html).*
///
/// Sometimes most of the fields of a GraphQL object simply return the value of the structure member, but a few
/// fields are calculated. Usually we use the `Object` macro to define such a GraphQL object.
///
/// But this can be done more beautifully with the `ComplexObject` macro. We can use the `SimpleObject` macro to define
/// some simple fields, and use the `ComplexObject` macro to define some other fields that need to be calculated.
///
/// # Macro parameters
///
/// | Attribute     | description               | Type     | Optional |
/// |---------------|---------------------------|----------|----------|
/// | name          | Object name               | string   | Y        |
/// | rename_fields | Rename all the fields according to the given case convention. The possible values are "lowercase", "UPPERCASE", "PascalCase", "camelCase", "snake_case", "SCREAMING_SNAKE_CASE".| string   | Y        |
/// | rename_args   | Rename all the arguments according to the given case convention. The possible values are "lowercase", "UPPERCASE", "PascalCase", "camelCase", "snake_case", "SCREAMING_SNAKE_CASE".| string   | Y        |
///
/// # Field parameters
///
/// | Attribute     | description               | Type     | Optional |
/// |---------------|---------------------------|----------|----------|
/// | skip          | Skip this field           | bool     | Y        |
/// | name          | Field name                | string   | Y        |
/// | deprecation   | Field deprecated          | bool     | Y        |
/// | deprecation   | Field deprecation reason  | string   | Y        |
/// | derived      | Generate derived fields *[See also the Book](https://async-graphql.github.io/async-graphql/en/derived_fields.html).*                 | object        | Y        |
/// | cache_control | Field cache control       | [`CacheControl`](struct.CacheControl.html) | Y        |
/// | external      | Mark a field as owned by another service. This allows service A to use fields from service B while also knowing at runtime the types of that field. | bool | Y |
/// | provides      | Annotate the expected returned fieldset from a field on a base type that is guaranteed to be selectable by the gateway. | string | Y |
/// | requires      | Annotate the required input fieldset from a base type for a resolver. It is used to develop a query plan where the required fields may not be needed by the client, but the service may need additional information from other services. | string | Y |
/// | guard         | Field of guard            | [`Guard`](guard/trait.Guard.html) | Y        |
/// | visible       | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool | Y |
/// | visible       | Call the specified function. If the return value is `false`, it will not be displayed in introspection. | string | Y |
/// | secret        | Mark this field as a secret, it will not output the actual value in the log. | bool | Y |
///
/// # Derived argument parameters
///
/// | Attribute    | description                              | Type        | Optional |
/// |--------------|------------------------------------------|------------ |----------|
/// | name         | Generated derived field name             | string      | N        |
/// | into         | Type to derived an into                  | string      | Y        |
/// | with         | Function to apply to manage advanced use cases | string| Y        |
///
/// # Examples
///
/// ```rust
/// use async_graphql::*;
///
/// #[derive(SimpleObject)]
/// #[graphql(complex)] // NOTE: If you want the `ComplexObject` macro to take effect, this `complex` attribute is required.
/// struct MyObj {
///     a: i32,
///     b: i32,
/// }
///
/// #[ComplexObject]
/// impl MyObj {
///     async fn c(&self) -> i32 {
///         self.a + self.b     
///     }
/// }
///
/// struct QueryRoot;
///
/// #[Object]
/// impl QueryRoot {
///     async fn obj(&self) -> MyObj {
///         MyObj { a: 10, b: 20 }
///     }
/// }
///
/// tokio::runtime::Runtime::new().unwrap().block_on(async move {
///     let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
///     let res = schema.execute("{ obj { a b c } }").await.into_result().unwrap().data;
///     assert_eq!(res, value!({
///         "obj": {
///             "a": 10,
///             "b": 20,
///             "c": 30,
///         },
///     }));
/// });
/// ```
pub use async_graphql_derive::ComplexObject;

/// Define a GraphQL enum
///
/// *[See also the Book](https://async-graphql.github.io/async-graphql/en/define_enum.html).*
///
/// # Macro parameters
///
/// | Attribute    | description               | Type     | Optional |
/// |--------------|---------------------------|----------|----------|
/// | name         | Enum name                 | string   | Y        |
/// | rename_items | Rename all the fields according to the given case convention. The possible values are "lowercase", "UPPERCASE", "PascalCase", "camelCase", "snake_case", "SCREAMING_SNAKE_CASE".| string   | Y        |
/// | remote       | Derive a remote enum      | string   | Y        |
/// | visible       | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool | Y |
/// | visible       | Call the specified function. If the return value is `false`, it will not be displayed in introspection. | string | Y |
///
/// # Item parameters
///
/// | Attribute   | description               | Type     | Optional |
/// |-------------|---------------------------|----------|----------|
/// | name        | Item name                 | string   | Y        |
/// | deprecation | Item deprecated           | bool     | Y        |
/// | deprecation | Item deprecation reason   | string   | Y        |
/// | visible       | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool | Y |
/// | visible       | Call the specified function. If the return value is `false`, it will not be displayed in introspection. | string | Y |
///
/// # Examples
///
/// ```rust
/// use async_graphql::*;
///
/// #[derive(Enum, Copy, Clone, Eq, PartialEq)]
/// enum MyEnum {
///     A,
///     #[graphql(name = "b")] B,
/// }
///
/// struct QueryRoot {
///     value1: MyEnum,
///     value2: MyEnum,
/// }
///
/// #[Object]
/// impl QueryRoot {
///     /// value1
///     async fn value1(&self) -> MyEnum {
///         self.value1
///     }
///
///     /// value2
///     async fn value2(&self) -> MyEnum {
///         self.value2
///     }
/// }
///
/// tokio::runtime::Runtime::new().unwrap().block_on(async move {
///     let schema = Schema::new(QueryRoot{ value1: MyEnum::A, value2: MyEnum::B }, EmptyMutation, EmptySubscription);
///     let res = schema.execute("{ value1 value2 }").await.into_result().unwrap().data;
///     assert_eq!(res, value!({ "value1": "A", "value2": "b" }));
/// });
/// ```
pub use async_graphql_derive::Enum;

/// Define a GraphQL input object
///
/// *[See also the Book](https://async-graphql.github.io/async-graphql/en/define_input_object.html).*
///
/// # Macro parameters
///
/// | Attribute     | description               | Type     | Optional |
/// |---------------|---------------------------|----------|----------|
/// | name          | Object name               | string   | Y        |
/// | rename_fields | Rename all the fields according to the given case convention. The possible values are "lowercase", "UPPERCASE", "PascalCase", "camelCase", "snake_case", "SCREAMING_SNAKE_CASE".| string   | Y        |
/// | visible       | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool | Y |
/// | visible       | Call the specified function. If the return value is `false`, it will not be displayed in introspection. | string | Y |
///
/// # Field parameters
///
/// | Attribute    | description                              | Type        | Optional |
/// |--------------|------------------------------------------|-------------|----------|
/// | name         | Field name                               | string      | Y        |
/// | default      | Use `Default::default` for default value | none        | Y        |
/// | default      | Argument default value                   | literal     | Y        |
/// | default_with | Expression to generate default value     | code string | Y        |
/// | validator    | Input value validator                    | [`InputValueValidator`](validators/trait.InputValueValidator.html) | Y        |
/// | flatten      | Similar to serde (flatten)               | boolean     | Y        |
/// | skip         | Skip this field, use `Default::default` to get a default value for this field. | bool     | Y        |
/// | visible      | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool | Y |
/// | visible      | Call the specified function. If the return value is `false`, it will not be displayed in introspection. | string | Y |
/// | secret       | Mark this field as a secret, it will not output the actual value in the log. | bool | Y |
///
/// # Examples
///
/// ```rust
/// use async_graphql::*;
///
/// #[derive(InputObject)]
/// struct MyInputObject {
///     a: i32,
///     #[graphql(default = 10)]
///     b: i32,
/// }
///
/// struct QueryRoot;
///
/// #[Object]
/// impl QueryRoot {
///     /// value
///     async fn value(&self, input: MyInputObject) -> i32 {
///         input.a * input.b
///     }
/// }
///
/// tokio::runtime::Runtime::new().unwrap().block_on(async move {
///     let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
///     let res = schema.execute(r#"
///     {
///         value1: value(input:{a:9, b:3})
///         value2: value(input:{a:9})
///     }"#).await.into_result().unwrap().data;
///     assert_eq!(res, value!({ "value1": 27, "value2": 90 }));
/// });
/// ```
pub use async_graphql_derive::InputObject;

/// Define a GraphQL interface
///
/// *[See also the Book](https://async-graphql.github.io/async-graphql/en/define_interface.html).*
///
/// # Macro parameters
///
/// | Attribute     | description               | Type     | Optional |
/// |---------------|---------------------------|----------|----------|
/// | name          | Object name               | string   | Y        |
/// | rename_fields | Rename all the fields according to the given case convention. The possible values are "lowercase", "UPPERCASE", "PascalCase", "camelCase", "snake_case", "SCREAMING_SNAKE_CASE".| string   | Y        |
/// | rename_args   | Rename all the arguments according to the given case convention. The possible values are "lowercase", "UPPERCASE", "PascalCase", "camelCase", "snake_case", "SCREAMING_SNAKE_CASE".| string   | Y        |
/// | field         | Fields of this Interface  | InterfaceField | N |
/// | extends       | Add fields to an entity that's defined in another service | bool | Y |
/// | visible       | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool | Y |
/// | visible       | Call the specified function. If the return value is `false`, it will not be displayed in introspection. | string | Y |
///
/// # Field parameters
///
/// | Attribute   | description               | Type     | Optional |
/// |-------------|---------------------------|----------|----------|
/// | name        | Field name                | string   | N        |
/// | type        | Field type                | string   | N        |
/// | method      | Rust resolver method name. If specified, `name` will not be camelCased in schema definition | string | Y |
/// | desc        | Field description         | string   | Y        |
/// | deprecation | Field deprecated          | bool     | Y        |
/// | deprecation | Field deprecation reason  | string   | Y        |
/// | arg         | Field arguments           | InterfaceFieldArgument          | Y        |
/// | external    | Mark a field as owned by another service. This allows service A to use fields from service B while also knowing at runtime the types of that field. | bool | Y |
/// | provides    | Annotate the expected returned fieldset from a field on a base type that is guaranteed to be selectable by the gateway. | string | Y |
/// | requires    | Annotate the required input fieldset from a base type for a resolver. It is used to develop a query plan where the required fields may not be needed by the client, but the service may need additional information from other services. | string | Y |
/// | visible       | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool | Y |
/// | visible       | Call the specified function. If the return value is `false`, it will not be displayed in introspection. | string | Y |
///
/// # Field argument parameters
///
/// | Attribute    | description                              | Type        | Optional |
/// |--------------|------------------------------------------|-------------|----------|
/// | name         | Argument name                            | string      | N        |
/// | type         | Argument type                            | string      | N        |
/// | desc         | Argument description                     | string      | Y        |
/// | default      | Use `Default::default` for default value | none        | Y        |
/// | default      | Argument default value                   | literal     | Y        |
/// | default_with | Expression to generate default value     | code string | Y        |
/// | visible       | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool | Y |
/// | visible       | Call the specified function. If the return value is `false`, it will not be displayed in introspection. | string | Y |
/// | secret       | Mark this field as a secret, it will not output the actual value in the log. | bool | Y |
///
/// # Define an interface
///
/// Define TypeA, TypeB, TypeC... Implement the MyInterface
///
/// ```ignore
/// #[derive(Interface)]
/// enum MyInterface {
///     TypeA(TypeA),
///     TypeB(TypeB),
///     TypeC(TypeC),
///     ...
/// }
/// ```
///
/// # Fields
///
/// The type, name, and parameter fields of the interface must exactly match the type of the
/// implementation interface, but Result can be omitted.
///
/// ```rust
/// use async_graphql::*;
///
/// struct TypeA {
///     value: i32,
/// }
///
/// #[Object]
/// impl TypeA {
///     /// Returns data borrowed from the context
///     async fn value_a<'a>(&self, ctx: &'a Context<'_>) -> Result<&'a str> {
///         Ok(ctx.data::<String>()?.as_str())
///     }
///
///     /// Returns data borrowed self
///     async fn value_b(&self) -> &i32 {
///         &self.value
///     }
///
///     /// With parameters
///     async fn value_c(&self, a: i32, b: i32) -> i32 {
///         a + b
///     }
///
///     /// Disabled name transformation, don't forget "method" argument in interface!
///     #[graphql(name = "value_d")]
///     async fn value_d(&self) -> i32 {
///         &self.value + 1
///     }
/// }
///
/// #[derive(Interface)]
/// #[graphql(
///     field(name = "value_a", type = "&'ctx str"),
///     field(name = "value_b", type = "&i32"),
///     field(name = "value_c", type = "i32",
///         arg(name = "a", type = "i32"),
///         arg(name = "b", type = "i32")),
///     field(name = "value_d", method = "value_d", type = "i32"),
/// )]
/// enum MyInterface {
///     TypeA(TypeA)
/// }
///
/// struct QueryRoot;
///
/// #[Object]
/// impl QueryRoot {
///     async fn type_a(&self) -> MyInterface {
///         TypeA { value: 10 }.into()
///     }
/// }
///
/// tokio::runtime::Runtime::new().unwrap().block_on(async move {
///     let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).data("hello".to_string()).finish();
///     let res = schema.execute(r#"
///     {
///         typeA {
///             valueA
///             valueB
///             valueC(a: 3, b: 2)
///             value_d
///         }
///     }"#).await.into_result().unwrap().data;
///     assert_eq!(res, value!({
///         "typeA": {
///             "valueA": "hello",
///             "valueB": 10,
///             "valueC": 5,
///             "value_d": 11
///         }
///     }));
/// });
/// ```
pub use async_graphql_derive::Interface;

/// Define a GraphQL union
///
/// *[See also the Book](https://async-graphql.github.io/async-graphql/en/define_union.html).*
///
/// # Macro parameters
///
/// | Attribute   | description               | Type     | Optional |
/// |-------------|---------------------------|----------|----------|
/// | name        | Object name               | string   | Y        |
/// | visible       | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool | Y |
/// | visible       | Call the specified function. If the return value is `false`, it will not be displayed in introspection. | string | Y |
///
/// # Item parameters
///
/// | Attribute    | description                              | Type     | Optional |
/// |--------------|------------------------------------------|----------|----------|
/// | flatten      | Similar to serde (flatten)               | boolean  | Y        |
/// | visible       | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool | Y |
/// | visible       | Call the specified function. If the return value is `false`, it will not be displayed in introspection. | string | Y |
///
/// # Define a union
///
/// Define TypeA, TypeB, ... as MyUnion
///
/// ```rust
/// use async_graphql::*;
///
/// #[derive(SimpleObject)]
/// struct TypeA {
///     value_a: i32,
/// }
///
/// #[derive(SimpleObject)]
/// struct TypeB {
///     value_b: i32
/// }
///
/// #[derive(Union)]
/// enum MyUnion {
///     TypeA(TypeA),
///     TypeB(TypeB),
/// }
///
/// struct QueryRoot;
///
/// #[Object]
/// impl QueryRoot {
///     async fn all_data(&self) -> Vec<MyUnion> {
///         vec![TypeA { value_a: 10 }.into(), TypeB { value_b: 20 }.into()]
///     }
/// }
///
/// tokio::runtime::Runtime::new().unwrap().block_on(async move {
///     let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).data("hello".to_string()).finish();
///     let res = schema.execute(r#"
///     {
///         allData {
///             ... on TypeA {
///                 valueA
///             }
///             ... on TypeB {
///                 valueB
///             }
///         }
///     }"#).await.into_result().unwrap().data;
///     assert_eq!(res, value!({
///         "allData": [
///             { "valueA": 10 },
///             { "valueB": 20 },
///         ]
///     }));
/// });
/// ```
pub use async_graphql_derive::Union;

/// Define a GraphQL subscription
///
/// *[See also the Book](https://async-graphql.github.io/async-graphql/en/subscription.html).*
///
/// The field function is a synchronization function that performs filtering. When true is returned, the message is pushed to the client.
/// The second parameter is the type of the field.
/// Starting with the third parameter is one or more filtering conditions, The filter condition is the parameter of the field.
/// The filter function should be synchronous.
///
/// # Macro parameters
///
/// | Attribute     | description               | Type     | Optional |
/// |---------------|---------------------------|----------|----------|
/// | name          | Object name               | string   | Y        |
/// | rename_fields | Rename all the fields according to the given case convention. The possible values are "lowercase", "UPPERCASE", "PascalCase", "camelCase", "snake_case", "SCREAMING_SNAKE_CASE".| string   | Y        |
/// | rename_args   | Rename all the arguments according to the given case convention. The possible values are "lowercase", "UPPERCASE", "PascalCase", "camelCase", "snake_case", "SCREAMING_SNAKE_CASE".| string   | Y        |
/// | extends       | Add fields to an entity that's defined in another service | bool | Y |
/// | use_type_description | Specifies that the description of the type is on the type declaration. [`Description`]()(derive.Description.html) | bool | Y |
///
/// # Field parameters
///
/// | Attribute   | description               | Type     | Optional |
/// |-------------|---------------------------|----------|----------|
/// | name        | Field name                | string   | Y        |
/// | deprecation | Field deprecated          | bool     | Y        |
/// | deprecation | Field deprecation reason  | string   | Y        |
/// | guard       | Field of guard            | [`Guard`](guard/trait.Guard.html) | Y        |
/// | visible       | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool | Y |
/// | visible       | Call the specified function. If the return value is `false`, it will not be displayed in introspection. | string | Y |
/// | secret       | Mark this field as a secret, it will not output the actual value in the log. | bool | Y |
///
/// # Field argument parameters
///
/// | Attribute    | description                              | Type        | Optional |
/// |--------------|------------------------------------------|------------ |----------|
/// | name         | Argument name                            | string      | Y        |
/// | desc         | Argument description                     | string      | Y        |
/// | default      | Use `Default::default` for default value | none        | Y        |
/// | default      | Argument default value                   | literal     | Y        |
/// | default_with | Expression to generate default value     | code string | Y        |
/// | validator    | Input value validator                    | [`InputValueValidator`](validators/trait.InputValueValidator.html) | Y        |
/// | visible       | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool | Y |
/// | visible       | Call the specified function. If the return value is `false`, it will not be displayed in introspection. | string | Y |
///
/// # Examples
///
/// ```rust
/// use async_graphql::*;
/// use futures_util::stream::{Stream, StreamExt};
///
/// struct SubscriptionRoot;
///
/// #[Subscription]
/// impl SubscriptionRoot {
///     async fn value(&self, condition: i32) -> impl Stream<Item = i32> {
///         // Returns the number from 0 to `condition`.
///         futures_util::stream::iter(0..condition)
///     }
/// }
/// ```
pub use async_graphql_derive::Subscription;

/// Define a Scalar
///
/// # Macro parameters
///
/// | Attribute   | description               | Type     | Optional |
/// |-------------|---------------------------|----------|----------|
/// | name        | Scalar name               | string   | Y        |
/// | specified_by_url | Provide a specification URL for this scalar type, it must link to a human-readable specification of the data format, serialization and coercion rules for this scalar. | string | Y |
///
pub use async_graphql_derive::Scalar;

/// Define a NewType Scalar
///
/// It also implements `From<InnerType>` and `Into<InnerType>`.
///
/// # Macro parameters
///
/// | Attribute   | description               | Type     | Optional |
/// |-------------|---------------------------|----------|----------|
/// | name        | If this attribute is provided then define a new scalar, otherwise it is just a transparent proxy for the internal scalar. | string   | Y      |
/// | name        | If this attribute is provided then define a new scalar, otherwise it is just a transparent proxy for the internal scalar. | bool   | Y        |
/// | visible(Only valid for new scalars)   | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool | Y |
/// | visible(Only valid for new scalars)   | Call the specified function. If the return value is `false`, it will not be displayed in introspection. | string | Y |
/// | specified_by_url(Only valid for new scalars) | Provide a specification URL for this scalar type, it must link to a human-readable specification of the data format, serialization and coercion rules for this scalar. | string | Y |
///
/// # Examples
///
/// ## Use the original scalar name
///
/// ```rust
/// use async_graphql::*;
///
/// #[derive(NewType)]
/// struct Weight(f64);
///
/// struct QueryRoot;
///
/// #[Object]
/// impl QueryRoot {
///     async fn value(&self) -> Weight {
///         Weight(1.234)
///     }
/// }
///
/// // Test conversion
/// let weight: Weight = 10f64.into();
/// let weight_f64: f64 = weight.into();
///
/// tokio::runtime::Runtime::new().unwrap().block_on(async move {
///     let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).data("hello".to_string()).finish();
///
///     let res = schema.execute("{ value }").await.into_result().unwrap().data;
///     assert_eq!(res, value!({
///         "value": 1.234,
///     }));
///
///     let res = schema.execute(r#"
///     {
///         __type(name: "QueryRoot") {
///             fields {
///                 name type {
///                     kind
///                     ofType { name }
///                 }
///             }
///         }
///     }"#).await.into_result().unwrap().data;
///     assert_eq!(res, value!({
///         "__type": {
///             "fields": [{
///                 "name": "value",
///                 "type": {
///                     "kind": "NON_NULL",
///                     "ofType": {
///                         "name": "Float"
///                     }
///                 }
///             }]
///         }
///     }));
/// });
/// ```
///
/// ## Define a new scalar
///
/// ```rust
/// use async_graphql::*;
///
/// /// Widget NewType
/// #[derive(NewType)]
/// #[graphql(name)] // or: #[graphql(name = true)], #[graphql(name = "Weight")]
/// struct Weight(f64);
///
/// struct QueryRoot;
///
/// #[Object]
/// impl QueryRoot {
///     async fn value(&self) -> Weight {
///         Weight(1.234)
///     }
/// }
///
/// tokio::runtime::Runtime::new().unwrap().block_on(async move {
///     let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).data("hello".to_string()).finish();
///
///     let res = schema.execute("{ value }").await.into_result().unwrap().data;
///     assert_eq!(res, value!({
///         "value": 1.234,
///     }));
///
///     let res = schema.execute(r#"
///     {
///         __type(name: "QueryRoot") {
///             fields {
///                 name type {
///                     kind
///                     ofType { name }
///                 }
///             }
///         }
///     }"#).await.into_result().unwrap().data;
///     assert_eq!(res, value!({
///         "__type": {
///             "fields": [{
///                 "name": "value",
///                 "type": {
///                     "kind": "NON_NULL",
///                     "ofType": {
///                         "name": "Weight"
///                     }
///                 }
///             }]
///         }
///     }));
///
///     assert_eq!(schema.execute(r#"{ __type(name: "Weight") { name description } }"#).
///         await.into_result().unwrap().data, value!({
///             "__type": {
///                 "name": "Weight", "description": "Widget NewType"
///             }
///         }));
/// });
/// ```
pub use async_graphql_derive::NewType;

/// Define a merged object with multiple object types.
///
/// *[See also the Book](https://async-graphql.github.io/async-graphql/en/merging_objects.html).*
///
/// # Macro parameters
///
/// | Attribute     | description               | Type     | Optional |
/// |---------------|---------------------------|----------|----------|
/// | name          | Object name               | string   | Y        |
/// | cache_control | Object cache control      | [`CacheControl`](struct.CacheControl.html) | Y        |
/// | extends       | Add fields to an entity that's defined in another service | bool | Y |
/// | visible       | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool | Y |
/// | visible       | Call the specified function. If the return value is `false`, it will not be displayed in introspection. | string | Y |
/// | serial        | Resolve each field sequentially.         | bool        | Y        |
///
/// # Examples
///
/// ```rust
/// use async_graphql::*;
///
/// #[derive(SimpleObject)]
///  struct Object1 {
///     a: i32,
///  }
///
/// #[derive(SimpleObject)]
/// struct Object2 {
///     b: i32,
/// }
///
/// #[derive(SimpleObject)]
/// struct Object3 {
///     c: i32,
/// }
///
/// #[derive(MergedObject)]
/// struct MyObj(Object1, Object2, Object3);
///
/// let obj = MyObj(Object1 { a: 10 }, Object2 { b: 20 }, Object3 { c: 30 });
/// ```
pub use async_graphql_derive::MergedObject;

/// Define a merged subscription with multiple subscription types.
///
/// *[See also the Book](https://async-graphql.github.io/async-graphql/en/merging_objects.html).*
///
/// # Macro parameters
///
/// | Attribute     | description               | Type     | Optional |
/// |---------------|---------------------------|----------|----------|
/// | name          | Object name               | string   | Y        |
/// | extends       | Add fields to an entity that's defined in another service | bool | Y |
/// | visible       | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool | Y |
/// | visible       | Call the specified function. If the return value is `false`, it will not be displayed in introspection. | string | Y |
///
/// # Examples
///
/// ```rust
/// use async_graphql::*;
/// use futures_util::stream::Stream;
///
/// #[derive(Default)]
/// struct Subscription1;
///
/// #[Subscription]
/// impl Subscription1 {
///     async fn events1(&self) -> impl Stream<Item = i32> {
///         futures_util::stream::iter(0..10)
///     }
/// }
///
/// #[derive(Default)]
/// struct Subscription2;
///
/// #[Subscription]
/// impl Subscription2 {
///     async fn events2(&self) -> impl Stream<Item = i32> {
///         futures_util::stream::iter(10..20)
///    }
/// }
///
/// #[derive(MergedSubscription, Default)]
/// struct Subscription(Subscription1, Subscription2);
/// ```
pub use async_graphql_derive::MergedSubscription;

/// Attach a description to `Object`, `Scalar` or `Subscription`.
///
/// The three types above use the rustdoc on the implementation block as
/// the GraphQL type description, but if you want to use the rustdoc on the
/// type declaration as the GraphQL type description, you can use that derived macro.
///
/// # Examples
///
/// ```rust
/// use async_graphql::*;
///
/// /// This is MyObj
/// #[derive(Description, Default)]
/// struct MyObj;
///
/// #[Object(use_type_description)]
/// impl MyObj {
///     async fn value(&self) -> i32 {
///         100
///     }
/// }
///
/// #[derive(SimpleObject, Default)]
/// struct Query {
///     obj: MyObj,
/// }
///
/// tokio::runtime::Runtime::new().unwrap().block_on(async move {
///     let schema = Schema::new(Query::default(), EmptyMutation, EmptySubscription);
///     assert_eq!(
///         schema
///             .execute(r#"{ __type(name: "MyObj") { description } }"#)
///             .await
///             .data,
///         value!({
///             "__type": { "description": "This is MyObj" }
///         })
///     );
/// });
/// ```
pub use async_graphql_derive::Description;
