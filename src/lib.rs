//! # The GraphQL server library implemented by rust
//!
//! <div align="center">
//! <!-- CI -->
//! <img src="https://github.com/async-graphql/async-graphql/workflows/CI/badge.svg" />
//! <!-- codecov -->
//  <img src="https://codecov.io/gh/async-graphql/async-graphql/branch/master/graph/badge.svg" />
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
//! </div>
//!
//! ## Documentation
//!
//! * [Feature Comparison](feature-comparison.md)
//! * [Book](https://async-graphql.github.io/async-graphql/en/index.html)
//! * [中文文档](https://async-graphql.github.io/async-graphql/zh-CN/index.html)
//! * [Docs](https://docs.rs/async-graphql)
//! * [GitHub repository](https://github.com/async-graphql/async-graphql)
//! * [Cargo package](https://crates.io/crates/async-graphql)
//! * Minimum supported Rust version: 1.42 or later
//!
//! ## Features
//!
//! * Fully support async/await
//! * Type safety
//! * Rustfmt friendly (Procedural Macro)
//! * Custom scalar
//! * Minimal overhead
//! * Easy integration (hyper, actix_web, tide ...)
//! * Upload files (Multipart request)
//! * Subscription (WebSocket transport)
//! * Custom extension
//! * Apollo Tracing extension
//! * Limit query complexity/depth
//! * Error Extensions
//! * Apollo Federation
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
//! (./LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
//! * MIT license (./LICENSE-MIT or http://opensource.org/licenses/MIT)
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
//! If you are just getting started, we recommend checking out our examples at:
//! [https://github.com/async-graphql/examples](https://github.com/async-graphql/examples)
//!
//! To see how you would create a Relay-compliant server using async-graphql, warp, diesel & postgresql, you can also check out a real-world example at:
//! [https://github.com/phated/twentyfive-stars](https://github.com/phated/twentyfive-stars)
//!
//! ## Benchmarks
//!
//! ```shell script
//! git clone https://github.com/async-graphql/benchmark
//! cargo run --release
//! ```
//!

#![warn(missing_docs)]
#![allow(clippy::needless_doctest_main)]
#![allow(clippy::needless_lifetimes)]
#![recursion_limit = "256"]

#[macro_use]
extern crate thiserror;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

mod base;
mod context;
mod error;
mod look_ahead;
mod model;
mod mutation_resolver;
mod query;
mod resolver;
mod scalars;
mod schema;
mod subscription;
mod types;
mod validation;

pub mod extensions;
pub mod guard;
pub mod validators;

#[doc(hidden)]
pub use async_graphql_parser as parser;

#[doc(hidden)]
pub use anyhow;
#[doc(hidden)]
pub use async_trait;
#[doc(hidden)]
pub use futures;
#[doc(hidden)]
pub use indexmap;
#[doc(hidden)]
pub use serde_json;

pub mod http;

pub use base::{ScalarType, Type};
pub use context::{
    Context, ContextBase, Data, QueryEnv, QueryPathNode, QueryPathSegment, Variables,
};
pub use error::{
    Error, ErrorExtensions, FieldError, FieldResult, InputValueError, InputValueResult,
    ParseRequestError, QueryError, ResultExt,
};
pub use look_ahead::Lookahead;
pub use parser::{Pos, Positioned, Value};
pub use query::{
    IntoQueryBuilder, IntoQueryBuilderOpts, QueryBuilder, QueryResponse, StreamResponse,
};
pub use registry::CacheControl;
pub use scalars::{Any, Json, ID};
pub use schema::{Schema, SchemaBuilder, SchemaEnv};
pub use subscription::{
    SimpleBroker, SubscriptionStream, SubscriptionStreams, SubscriptionTransport,
    WebSocketTransport,
};
pub use types::{
    Connection, Cursor, DataSource, Deferred, EmptyEdgeFields, EmptyMutation, EmptySubscription,
    PageInfo, QueryOperation, Streamed, Upload,
};
pub use validation::ValidationMode;

/// Result type
pub type Result<T> = std::result::Result<T, Error>;

// internal types
#[doc(hidden)]
pub use context::ContextSelectionSet;
#[doc(hidden)]
pub mod registry;
#[doc(hidden)]
pub use base::{BoxFieldFuture, InputObjectType, InputValueType, ObjectType, OutputValueType};
#[doc(hidden)]
pub use resolver::{collect_fields, do_resolve};
#[doc(hidden)]
pub use subscription::SubscriptionType;
#[doc(hidden)]
pub use types::{EnumItem, EnumType};

/// Define a GraphQL object
///
/// # Macro parameters
///
/// | Attribute     | description               | Type     | Optional |
/// |---------------|---------------------------|----------|----------|
/// | name          | Object name               | string   | Y        |
/// | desc          | Object description        | string   | Y        |
/// | cache_control | Object cache control      | [`CacheControl`](struct.CacheControl.html) | Y        |
/// | extends       | Add fields to an entity that's defined in another service | bool | Y |
///
/// # Field parameters
///
/// | Attribute     | description               | Type     | Optional |
/// |---------------|---------------------------|----------|----------|
/// | name          | Field name                | string   | Y        |
/// | desc          | Field description         | string   | Y        |
/// | deprecation   | Field deprecation reason  | string   | Y        |
/// | cache_control | Field cache control       | [`CacheControl`](struct.CacheControl.html) | Y        |
/// | external      | Mark a field as owned by another service. This allows service A to use fields from service B while also knowing at runtime the types of that field. | bool | Y |
/// | provides      | Annotate the expected returned fieldset from a field on a base type that is guaranteed to be selectable by the gateway. | string | Y |
/// | requires      | Annotate the required input fieldset from a base type for a resolver. It is used to develop a query plan where the required fields may not be needed by the client, but the service may need additional information from other services. | string | Y |
/// | guard         | Field of guard            | [`Guard`](guard/trait.Guard.html) | Y        |
///
/// # Field argument parameters
///
/// | Attribute   | description               | Type     | Optional |
/// |-------------|---------------------------|----------|----------|
/// | name        | Argument name             | string   | Y        |
/// | desc        | Argument description      | string   | Y        |
/// | default     | Argument default value    | string   | Y        |
/// | validator   | Input value validator     | [`InputValueValidator`](validators/trait.InputValueValidator.html) | Y        |
///
/// # The field returns the value type
///
/// - A scalar value, such as `i32`, `bool`
/// - Borrowing of scalar values, such as `&i32`, `&bool`
/// - Vec<T>, such as `Vec<i32>`
/// - Slice<T>, such as `&[i32]`
/// - Option<T>, such as `Option<i32>`
/// - Object and &Object
/// - Enum
/// - FieldResult<T, E>, such as `FieldResult<i32, E>`
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
/// ```rust
/// use async_graphql::*;
///
/// struct QueryRoot {
///     value: i32,
/// }
///
/// #[Object]
/// impl QueryRoot {
///     #[field(desc = "value")]
///     async fn value(&self) -> i32 {
///         self.value
///     }
///
///     #[field(desc = "reference value")]
///     async fn value_ref(&self) -> &i32 {
///         &self.value
///     }
///
///     #[field(desc = "value with error")]
///     async fn value_with_error(&self) -> FieldResult<i32> {
///         Ok(self.value)
///     }
///
///     async fn value_with_arg(&self, #[arg(default = "1")] a: i32) -> i32 {
///         a
///     }
/// }
///
/// #[async_std::main]
/// async fn main() {
///     let schema = Schema::new(QueryRoot{ value: 10 }, EmptyMutation, EmptySubscription);
///     let res = schema.execute(r#"{
///         value
///         valueRef
///         valueWithError
///         valueWithArg1: valueWithArg
///         valueWithArg2: valueWithArg(a: 99)
///     }"#).await.unwrap().data;
///     assert_eq!(res, serde_json::json!({
///         "value": 10,
///         "valueRef": 10,
///         "valueWithError": 10,
///         "valueWithArg1": 1,
///         "valueWithArg2": 99
///     }));
/// }
/// ```
pub use async_graphql_derive::Object;

/// Define a GraphQL object
///
/// Similar to `Object`, but defined on a structure that automatically generates getters for all fields.
///
/// # Macro parameters
///
/// | Attribute     | description               | Type     | Optional |
/// |---------------|---------------------------|----------|----------|
/// | name          | Object name               | string   | Y        |
/// | desc          | Object description        | string   | Y        |
/// | cache_control | Object cache control      | [`CacheControl`](struct.CacheControl.html) | Y        |
///
/// # Field parameters
///
/// | Attribute     | description               | Type     | Optional |
/// |---------------|---------------------------|----------|----------|
/// | name          | Field name                | string   | Y        |
/// | desc          | Field description         | string   | Y        |
/// | deprecation   | Field deprecation reason  | string   | Y        |
/// | cache_control | Field cache control       | [`CacheControl`](struct.CacheControl.html) | Y        |
/// | external      | Mark a field as owned by another service. This allows service A to use fields from service B while also knowing at runtime the types of that field. | bool | Y |
/// | provides      | Annotate the expected returned fieldset from a field on a base type that is guaranteed to be selectable by the gateway. | string | Y |
/// | requires      | Annotate the required input fieldset from a base type for a resolver. It is used to develop a query plan where the required fields may not be needed by the client, but the service may need additional information from other services. | string | Y |
/// | guard         | Field of guard            | [`Guard`](guard/trait.Guard.html) | Y        |
///
/// # Examples
///
/// ```rust
/// use async_graphql::*;
///
/// #[SimpleObject]
/// struct QueryRoot {
///     value: i32,
/// }
///
/// #[async_std::main]
/// async fn main() {
///     let schema = Schema::new(QueryRoot{ value: 10 }, EmptyMutation, EmptySubscription);
///     let res = schema.execute("{ value }").await.unwrap().data;
///     assert_eq!(res, serde_json::json!({
///         "value": 10,
///     }));
/// }
/// ```
pub use async_graphql_derive::SimpleObject;

/// Define a GraphQL enum
///
/// # Macro parameters
///
/// | Attribute   | description               | Type     | Optional |
/// |-------------|---------------------------|----------|----------|
/// | name        | Enum name                 | string   | Y        |
/// | desc        | Enum description          | string   | Y        |
///
/// # Item parameters
///
/// | Attribute   | description               | Type     | Optional |
/// |-------------|---------------------------|----------|----------|
/// | name        | Item name                 | string   | Y        |
/// | desc        | Item description          | string   | Y        |
/// | deprecation | Item deprecation reason   | string   | Y        |
/// | ref         | The resolver function returns a borrowing value  | bool   | Y        |
///
/// # Examples
///
/// ```rust
/// use async_graphql::*;
///
/// #[Enum]
/// enum MyEnum {
///     A,
///     #[item(name = "b")] B,
/// }
///
/// struct QueryRoot {
///     value1: MyEnum,
///     value2: MyEnum,
/// }
///
/// #[Object]
/// impl QueryRoot {
///     #[field(desc = "value")]
///     async fn value1(&self) -> MyEnum {
///         self.value1
///     }
///
///     #[field(desc = "value")]
///     async fn value2(&self) -> MyEnum {
///         self.value2
///     }
/// }
///
/// #[async_std::main]
/// async fn main() {
///     let schema = Schema::new(QueryRoot{ value1: MyEnum::A, value2: MyEnum::B }, EmptyMutation, EmptySubscription);
///     let res = schema.execute("{ value1 value2 }").await.unwrap().data;
///     assert_eq!(res, serde_json::json!({ "value1": "A", "value2": "b" }));
/// }
/// ```
pub use async_graphql_derive::Enum;

/// Define a GraphQL input object
///
/// # Macro parameters
///
/// | Attribute   | description               | Type     | Optional |
/// |-------------|---------------------------|----------|----------|
/// | name        | Object name               | string   | Y        |
/// | desc        | Object description        | string   | Y        |
///
/// # Field parameters
///
/// | Attribute   | description               | Type     | Optional |
/// |-------------|---------------------------|----------|----------|
/// | name        | Field name                | string   | Y        |
/// | desc        | Field description         | string   | Y        |
/// | default     | Field default value       | string   | Y        |
/// | validator   | Input value validator     | [`InputValueValidator`](validators/trait.InputValueValidator.html) | Y        |
///
/// # Examples
///
/// ```rust
/// use async_graphql::*;
///
/// #[InputObject]
/// struct MyInputObject {
///     a: i32,
///     #[field(default = "10")]
///     b: i32,
/// }
///
/// struct QueryRoot;
///
/// #[Object]
/// impl QueryRoot {
///     #[field(desc = "value")]
///     async fn value(&self, input: MyInputObject) -> i32 {
///         input.a * input.b
///     }
/// }
///
/// #[async_std::main]
/// async fn main() {
///     let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
///     let res = schema.execute(r#"
///     {
///         value1: value(input:{a:9, b:3})
///         value2: value(input:{a:9})
///     }"#).await.unwrap().data;
///     assert_eq!(res, serde_json::json!({ "value1": 27, "value2": 90 }));
/// }
/// ```
pub use async_graphql_derive::InputObject;

/// Define a GraphQL interface
///
/// # Macro parameters
///
/// | Attribute   | description               | Type     | Optional |
/// |-------------|---------------------------|----------|----------|
/// | name        | Object name               | string   | Y        |
/// | desc        | Object description        | string   | Y        |
///
/// # Field parameters
///
/// | Attribute   | description               | Type     | Optional |
/// |-------------|---------------------------|----------|----------|
/// | name        | Field name                | string   | N        |
/// | type        | Field type                | string   | N        |
/// | desc        | Field description         | string   | Y        |
/// | deprecation | Field deprecation reason  | string   | Y        |
/// | args        | Field arguments           |          | Y        |
///
/// # Field argument parameters
///
/// | Attribute   | description               | Type     | Optional |
/// |-------------|---------------------------|----------|----------|
/// | name        | Argument name             | string   | N        |
/// | type        | Argument type             | string   | N        |
/// | desc        | Argument description      | string   | Y        |
/// | default     | Argument default value    | string   | Y        |
///
/// # Define an interface
///
/// Define TypeA, TypeB, TypeC... Implement the MyInterface
///
/// ```ignore
/// #[Interface]
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
/// implementation interface, but FieldResult can be omitted.
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
///     async fn value_a<'a>(&self, ctx: &'a Context<'_>) -> &'a str {
///         ctx.data::<String>().as_str()
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
/// }
///
/// #[Interface(
///     field(name = "value_a", type = "&'ctx str"),
///     field(name = "value_b", type = "&i32"),
///     field(name = "value_c", type = "i32",
///         arg(name = "a", type = "i32"),
///         arg(name = "b", type = "i32")),
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
/// #[async_std::main]
/// async fn main() {
///     let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).data("hello".to_string()).finish();
///     let res = schema.execute(r#"
///     {
///         typeA {
///             valueA
///             valueB
///             valueC(a: 3, b: 2)
///         }
///     }"#).await.unwrap().data;
///     assert_eq!(res, serde_json::json!({
///         "typeA": {
///             "valueA": "hello",
///             "valueB": 10,
///             "valueC": 5
///         }
///     }));
/// }
/// ```
pub use async_graphql_derive::Interface;

/// Define a GraphQL union
///
///
/// # Macro parameters
///
/// | Attribute   | description               | Type     | Optional |
/// |-------------|---------------------------|----------|----------|
/// | name        | Object name               | string   | Y        |
/// | desc        | Object description        | string   | Y        |
///
/// # Define a union
///
/// Define TypeA, TypeB, ... as MyUnion
///
/// ```rust
/// use async_graphql::*;
///
/// #[SimpleObject]
/// struct TypeA {
///     value_a: i32,
/// }
///
/// #[SimpleObject]
/// struct TypeB {
///     value_b: i32
/// }
///
/// #[Union]
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
/// #[async_std::main]
/// async fn main() {
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
///     }"#).await.unwrap().data;
///     assert_eq!(res, serde_json::json!({
///         "allData": [
///             { "valueA": 10 },
///             { "valueB": 20 },
///         ]
///     }));
/// }
/// ```
pub use async_graphql_derive::Union;

/// Define a GraphQL subscription
///
/// The field function is a synchronization function that performs filtering. When true is returned, the message is pushed to the client.
/// The second parameter is the type of the field.
/// Starting with the third parameter is one or more filtering conditions, The filter condition is the parameter of the field.
/// The filter function should be synchronous.
///
/// # Macro parameters
///
/// | Attribute   | description               | Type     | Optional |
/// |-------------|---------------------------|----------|----------|
/// | name        | Object name               | string   | Y        |
/// | desc        | Object description        | string   | Y        |
///
/// # Field parameters
///
/// | Attribute   | description               | Type     | Optional |
/// |-------------|---------------------------|----------|----------|
/// | name        | Field name                | string   | Y        |
/// | desc        | Field description         | string   | Y        |
/// | deprecation | Field deprecation reason  | string   | Y        |
/// | guard         | Field of guard            | [`Guard`](guard/trait.Guard.html) | Y        |
///
/// # Field argument parameters
///
/// | Attribute   | description               | Type     | Optional |
/// |-------------|---------------------------|----------|----------|
/// | name        | Argument name             | string   | Y        |
/// | desc        | Argument description      | string   | Y        |
/// | default     | Argument default value    | string   | Y        |
/// | validator   | Input value validator     | [`InputValueValidator`](validators/trait.InputValueValidator.html) | Y        |
///
/// # Examples
///
/// ```ignore
/// use async_graphql::*;
///
/// #[Object]
/// struct Event {
///     value: i32,
/// }
///
/// struct SubscriptionRoot;
///
/// #[Subscription]
/// impl SubscriptionRoot {
///     async fn value(&self, event: &Event, condition: i32) -> bool {
///         // Push when value is greater than condition
///         event.value > condition
///     }
/// }
/// ```
pub use async_graphql_derive::Subscription;

/// Define a DataSource
pub use async_graphql_derive::DataSource;

/// Define a Scalar
///
/// # Macro parameters
///
/// | Attribute   | description               | Type     | Optional |
/// |-------------|---------------------------|----------|----------|
/// | name        | Scalar name               | string   | Y        |
/// | desc        | Scalar description        | string   | Y        |
///
pub use async_graphql_derive::Scalar;
