# Poem

## Request example

```rust
use poem::Route;
use async_graphql_poem::GraphQL;

let app = Route::new()
    .at("/ws", GraphQL::new(schema));
```

## Subscription example

```rust
use poem::Route;
use async_graphql_poem::GraphQLSubscription;

let app = Route::new()
    .at("/ws", get(GraphQLSubscription::new(schema)));
```

## More examples

[https://github.com/async-graphql/examples/tree/master/poem](https://github.com/async-graphql/examples/tree/master/poem)
