use async_graphql::{resolver_utils::ObjectType, Response, Schema, SubscriptionType};
use async_graphql_parser::{parse_query, types::ExecutableDocument};
use async_std::task;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

pub fn run<Query, Mutation, Subscription>(
    s: &Schema<Query, Mutation, Subscription>,
    q: &str,
) -> Response
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    task::block_on(async { s.execute(q).await.into_result().unwrap() })
}

pub fn parse(q: &str) -> ExecutableDocument {
    parse_query(q).unwrap()
}

// pub fn validate() {
//     check_rules(&S.env.registry, &D, S.validation_mode).unwrap();
// }
//
// pub fn resolve() {
//     do_resolve(...).unwrap();
// }

pub fn serialize(r: &async_graphql::Response) -> String {
    serde_json::to_string(r).unwrap()
}
