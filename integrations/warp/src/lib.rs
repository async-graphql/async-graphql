//! Async-graphql integration with Warp

#![allow(clippy::type_complexity)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod batch_request;
mod error;
mod request;
mod subscription;

pub use batch_request::{GraphQLBatchResponse, graphql_batch, graphql_batch_opts};
pub use error::GraphQLBadRequest;
pub use request::{GraphQLResponse, graphql, graphql_opts};
pub use subscription::{GraphQLWebSocket, graphql_protocol, graphql_subscription};
