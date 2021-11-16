//! Async-graphql integration with Actix-web
#![forbid(unsafe_code)]
#![allow(clippy::upper_case_acronyms)]
#![warn(missing_docs)]

mod request;
mod subscription;

pub use request::{GraphQLBatchRequest, GraphQLRequest, GraphQLResponse};
pub use subscription::GraphQLSubscription;
