//! Async-graphql integration with Axum
#![forbid(unsafe_code)]
#![allow(clippy::uninlined_format_args)]
#![warn(missing_docs)]

mod extract;
mod query;
mod response;
mod subscription;

pub use extract::{GraphQLBatchRequest, GraphQLRequest};
pub use query::GraphQL;
pub use response::GraphQLResponse;
pub use subscription::{GraphQLProtocol, GraphQLSubscription, GraphQLWebSocket};
