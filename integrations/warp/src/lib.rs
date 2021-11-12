//! Async-graphql integration with Warp

#![allow(clippy::type_complexity)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod batch_request;
mod error;
mod request;
mod subscription;

pub use batch_request::{graphql_batch, graphql_batch_opts, BatchResponse};
pub use error::BadRequest;
pub use request::{graphql, graphql_opts, Response};
pub use subscription::{graphql_protocol, graphql_subscription, GraphQLWebSocket};
