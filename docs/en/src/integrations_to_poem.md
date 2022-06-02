# Poem

## Request example

```rust
# extern crate async_graphql_poem;
# extern crate async_graphql;
# extern crate poem;
# use async_graphql::*;
# #[derive(Default, SimpleObject)]
# struct Query { a: i32 }
# let schema = Schema::build(Query::default(), EmptyMutation, EmptySubscription).finish();
use poem::Route;
use async_graphql_poem::GraphQL;

let app = Route::new()
    .at("/ws", GraphQL::new(schema));
```

## Subscription example

```rust
# extern crate async_graphql_poem;
# extern crate async_graphql;
# extern crate poem;
# use async_graphql::*;
# #[derive(Default, SimpleObject)]
# struct Query { a: i32 }
# let schema = Schema::build(Query::default(), EmptyMutation, EmptySubscription).finish();
use poem::{get, Route};
use async_graphql_poem::GraphQLSubscription;

let app = Route::new()
    .at("/ws", get(GraphQLSubscription::new(schema)));
```

## More examples

[https://github.com/async-graphql/examples/tree/master/poem](https://github.com/async-graphql/examples/tree/master/poem)
