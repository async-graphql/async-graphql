//! Async-graphql integration with Poem
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod extractor;
mod query;
mod subscription;

pub use extractor::{GraphQLBatchRequest, GraphQLRequest};
pub use query::GraphQL;
pub use subscription::GraphQLSubscription;
