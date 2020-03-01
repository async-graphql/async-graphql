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
//! ## References
//!
//! * [GraphQL](https://graphql.org)

#[macro_use]
extern crate thiserror;

mod context;
mod r#enum;
mod error;
mod id;
mod query;
mod scalar;
mod r#type;

#[cfg(feature = "chrono")]
mod datetime;
#[cfg(feature = "uuid")]
mod uuid;

pub use anyhow;
pub use async_trait;
pub use graphql_parser;
pub use serde_json;

pub use async_graphql_derive::{Enum, InputObject, Object};
pub use context::{Context, ContextField, ContextSelectionSet, Data, Variables};
pub use error::{ErrorWithPosition, PositionError, QueryError, QueryParseError};
pub use graphql_parser::query::Value;
pub use id::ID;
pub use query::QueryBuilder;
pub use scalar::Scalar;

// internal types
pub use r#enum::{GQLEnum, GQLEnumItem};
pub use r#type::{
    GQLEmptyMutation, GQLInputObject, GQLInputValue, GQLObject, GQLOutputValue, GQLType,
};

pub type Result<T> = anyhow::Result<T>;
pub type Error = anyhow::Error;
