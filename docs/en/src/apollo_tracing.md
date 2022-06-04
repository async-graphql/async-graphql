# Apollo Tracing

Apollo Tracing provides performance analysis results for each step of query. This is an extension to `Schema`, and the performance analysis results are stored in `QueryResponse`.

To enable the Apollo Tracing extension, add the extension when the `Schema` is created.

```rust
# extern crate async_graphql;
use async_graphql::*;
use async_graphql::extensions::ApolloTracing;

# struct Query;
# #[Object]
# impl Query { async fn version(&self) -> &str { "1.0" } }

let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
    .extension(ApolloTracing) // Enable ApolloTracing extension
    .finish();
```
