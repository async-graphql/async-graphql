//! Async-graphql integration with Axum
#![forbid(unsafe_code)]
#![allow(clippy::uninlined_format_args)]
#![warn(missing_docs)]

mod extract;
mod query;
mod response;
#[cfg(not(target_arch = "wasm32"))]
mod subscription;

pub use extract::{rejection, GraphQLBatchRequest, GraphQLRequest};
pub use query::GraphQL;
pub use response::GraphQLResponse;
#[cfg(not(target_arch = "wasm32"))]
pub use subscription::{GraphQLProtocol, GraphQLSubscription, GraphQLWebSocket};
