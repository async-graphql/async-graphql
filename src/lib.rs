//! # The GraphQL server library implemented by rust
//!
//! <div align="center">
//! <!-- CI -->
//! <img src="https://github.com/sunli829/potatonet/workflows/CI/badge.svg" />
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
//! * [GitHub repository](https://github.com/sunli829/async-graphql)
//! * [Cargo package](https://crates.io/crates/async-graphql)
//! * Minimum supported Rust version: 1.39 or later
//!
//! ## Features
//!
//! * Fully support async/await.
//! * Type safety.
//! * Rustfmt friendly (Procedural Macro).
//! * Custom scalar.
//! * Minimal overhead.
//! * Easy integration (hyper, actix_web, tide ...).
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

#[macro_use]
extern crate thiserror;
#[macro_use]
extern crate serde_derive;

mod base;
mod context;
mod error;
mod model;
mod resolver;
mod scalars;
mod schema;
mod types;
mod validation;

#[doc(hidden)]
pub use anyhow;
#[doc(hidden)]
pub use async_trait;
#[doc(hidden)]
pub use graphql_parser;
#[doc(hidden)]
pub use serde_json;

pub mod http;

pub use async_graphql_derive::Union;
pub use base::GQLScalar;
pub use context::{Context, Variables};
pub use error::{ErrorWithPosition, PositionError, QueryError, QueryParseError};
pub use graphql_parser::query::Value;
pub use scalars::ID;
pub use schema::{QueryBuilder, Schema};
pub use types::GQLEmptyMutation;

pub type Result<T> = anyhow::Result<T>;
pub type Error = anyhow::Error;

// internal types
#[doc(hidden)]
pub use context::ContextSelectionSet;
#[doc(hidden)]
pub mod registry;
#[doc(hidden)]
pub use base::{GQLInputObject, GQLInputValue, GQLObject, GQLOutputValue, GQLType};
#[doc(hidden)]
pub use context::ContextBase;
#[doc(hidden)]
pub use resolver::do_resolve;
#[doc(hidden)]
pub use types::{GQLEnum, GQLEnumItem};

/// Define a GraphQL object
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
///
/// # Field argument parameters
///
/// | Attribute   | description               | Type     | Optional |
/// |-------------|---------------------------|----------|----------|
/// | name        | Argument name             | string   | Y        |
/// | desc        | Argument description      | string   | Y        |
/// | default     | Argument default value    | string   | Y        |
///
/// # The field returns the value type
///
/// - A scalar value, such as `i32`, `bool`
/// - Borrowing of scalar values, such as `&i32`, `&bool`
/// - Vec<T>, such as `Vec<i32>`
/// - Slice<T>, such as `&[i32]`
/// - Option<T>, such as `Option<i32>`
/// - GQLObject and `&GQLObject`
/// - GQLEnum
/// - Result<T, E>, such as `Result<i32, E>`
///
/// # Context
///
/// You can define a context as an argument to a method, and the context should be the first argument to the method.
///
/// ```ignore
/// #[Object]
/// impl MyObject {
///     async fn value(&self, ctx: &Context<'_>) -> { ... }
/// }
/// ```
///
/// # Examples
///
/// ```rust
/// use async_graphql::*;
///
/// struct MyObject {
///     value: i32,
/// }
///
/// #[Object]
/// impl MyObject {
///     #[field(desc = "value")]
///     async fn value(&self) -> i32 {
///         self.value
///     }
///
///     #[field(name = "valueRef", desc = "reference value")]
///     async fn value_ref(&self) -> &i32 {
///         &self.value
///     }
///
///     #[field(name = "valueWithError", desc = "value with error")]
///     async fn value_with_error(&self) -> Result<i32> {
///         Ok(self.value)
///     }
///
///     #[field(name = "valueWithArg")]
///     async fn value_with_arg(&self, #[arg(default = "1")] a: i32) -> i32 {
///         a
///     }
/// }
///
/// #[async_std::main]
/// async fn main() {
///     let schema = Schema::new(MyObject{ value: 10 }, GQLEmptyMutation);
///     let res = schema.query(r#"{
///         value
///         valueRef
///         valueWithError
///         valueWithArg1: valueWithArg
///         valueWithArg2: valueWithArg(a: 99)
///     }"#).execute().await.unwrap();
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
/// struct MyObject {
///     value1: MyEnum,
///     value2: MyEnum,
/// }
///
/// #[Object]
/// impl MyObject {
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
///     let schema = Schema::new(MyObject{ value1: MyEnum::A, value2: MyEnum::B }, GQLEmptyMutation);
///     let res = schema.query("{ value1 value2 }").execute().await.unwrap();
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
/// struct MyObject;
///
/// #[Object]
/// impl MyObject {
///     #[field(desc = "value")]
///     async fn value(&self, input: MyInputObject) -> i32 {
///         input.a * input.b
///     }
/// }
///
/// #[async_std::main]
/// async fn main() {
///     let schema = Schema::new(MyObject, GQLEmptyMutation);
///     let res = schema.query(r#"
///     {
///         value1: value(input:{a:9, b:3})
///         value2: value(input:{a:9})
///     }"#).execute().await.unwrap();
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
/// | method      | Field method name         | string   | Y        |
/// | context     | Method with the context   | string   | Y        |
/// | deprecation | Field deprecation reason  | string   | Y        |
/// | args        | Field arguments           | [Arg]    | Y        |
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
/// struct MyInterface(TypeA, TypeB, TypeC, ...);
/// ```
///
/// # Fields
///
/// The type, name, and parameters of the interface field must exactly match the type that implements the interface,
/// The internal implementation is a forward of the function call.
/// You can specify the field function name that implements the interface type through the 'method' property,
/// or you can specify that the field function has a context parameter through the 'context' attribute.
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
///     #[field]
///     async fn value_a<'a>(&self, ctx: &'a Context<'_>) -> &'a str {
///         ctx.data::<String>().as_str()
///     }
///
///     /// Returns data borrowed self
///     #[field]
///     async fn value_b(&self) -> &i32 {
///         &self.value
///     }
///
///     /// With parameters
///     #[field]
///     async fn value_c(&self, a: i32, b: i32) -> i32 {
///         a + b
///     }
/// }
///
/// #[Interface(
///     field(name = "value_a", type = "&'ctx str", context),
///     field(name = "value_b", type = "&i32"),
///     field(name = "value_c", type = "i32",
///         arg(name = "a", type = "i32"),
///         arg(name = "b", type = "i32")),
/// )]
/// struct MyInterface(TypeA);
///
/// struct QueryRoot;
///
/// #[Object]
/// impl QueryRoot {
///     #[field]
///     async fn type_a(&self) -> MyInterface {
///         TypeA { value: 10 }.into()
///     }
/// }
///
/// #[async_std::main]
/// async fn main() {
///     let schema = Schema::new(QueryRoot, GQLEmptyMutation).data("hello".to_string());
///     let res = schema.query(r#"
///     {
///         type_a {
///             value_a
///             value_b
///             value_c(a: 3, b: 2)
///         }
///     }"#).execute().await.unwrap();
///     assert_eq!(res, serde_json::json!({
///         "type_a": {
///             "value_a": "hello",
///             "value_b": 10,
///             "value_c": 5
///         }
///     }));
/// }
/// ```
pub use async_graphql_derive::Interface;
