pub use async_graphql::http::GQLResponse;
use async_graphql::{ObjectType, QueryResponse, Schema, SubscriptionType};
use async_graphql_parser::{parse_query, query::Document};
use async_std::task;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

pub fn run<Query, Mutation, Subscription>(
    s: &Schema<Query, Mutation, Subscription>,
    q: &str,
) -> QueryResponse
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    task::block_on(async { s.execute(q).await.unwrap() })
}

pub fn parse(q: &str) -> Document {
    parse_query(q).unwrap()
}

// pub fn validate() {
//     check_rules(&S.env.registry, &D, S.validation_mode).unwrap();
// }
//
// pub fn resolve() {
//     do_resolve(...).unwrap();
// }

pub fn serialize(r: &GQLResponse) -> String {
    serde_json::to_string(&r).unwrap()
}
