//! Async-graphql integration with Axum
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod extract;
mod response;
mod subscription;

pub use extract::{GraphQLBatchRequest, GraphQLRequest};
pub use response::GraphQLResponse;
pub use subscription::{
    graphql_subscription, graphql_subscription_with_data, SecWebsocketProtocol,
};
