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

#[doc(hidden)]
pub use anyhow;
#[doc(hidden)]
pub use async_trait;
#[doc(hidden)]
pub use graphql_parser;
#[doc(hidden)]
pub use serde_json;

pub mod http;

pub use async_graphql_derive::{Enum, InputObject, Interface, Object};
pub use base::{GQLInputObject, GQLInputValue, GQLObject, GQLOutputValue, GQLScalar, GQLType};
pub use context::{Context, Variables};
pub use error::{ErrorWithPosition, PositionError, QueryError, QueryParseError};
pub use graphql_parser::query::Value;
pub use scalars::ID;
pub use schema::{QueryBuilder, Schema};
pub use types::GQLEmptyMutation;
pub use types::{GQLEnum, GQLEnumItem};

pub type Result<T> = anyhow::Result<T>;
pub type Error = anyhow::Error;

// internal types
#[doc(hidden)]
pub use context::ContextSelectionSet;
#[doc(hidden)]
pub mod registry;
#[doc(hidden)]
pub use context::ContextBase;
#[doc(hidden)]
pub use resolver::do_resolve;
