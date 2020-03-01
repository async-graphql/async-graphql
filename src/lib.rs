#[macro_use]
extern crate thiserror;
#[macro_use]
extern crate serde_derive;

mod context;
mod r#enum;
mod error;
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
pub use query::QueryBuilder;
pub use scalar::Scalar;

// internal types
pub use r#enum::{GQLEnum, GQLEnumItem};
pub use r#type::{
    GQLEmptyMutation, GQLInputObject, GQLInputValue, GQLObject, GQLOutputValue, GQLType,
};

pub type Result<T> = anyhow::Result<T>;
pub type Error = anyhow::Error;
