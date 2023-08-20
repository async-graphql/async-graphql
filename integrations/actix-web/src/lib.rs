//! Async-graphql integration with Actix-web
#![forbid(unsafe_code)]
#![allow(clippy::upper_case_acronyms)]
#![warn(missing_docs)]

mod handler;
mod request;
mod subscription;

pub use handler::GraphQL;
pub use request::{GraphQLBatchRequest, GraphQLRequest, GraphQLResponse};
pub use subscription::GraphQLSubscription;
