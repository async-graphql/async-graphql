# Apollo Tracing

Apollo Tracing provides performance analysis results for each step of query. This is an extension to `GqlSchema`, and the performance analysis results are stored in `QueryResponse`.

To enable the Apollo Tracing extension, add the extension when a `GqlSchema` is created.

```rust
use async_graphql::*;
use async_graphql::extensions::ApolloTracing;

let schema = GqlSchema::build(Query, EmptyMutation, EmptySubscription)
    .extension(|| ApolloTracing::default()) // Enable ApolloTracing extension
    .finish();

```