# Merging Objects

Usually we can create multiple implementations for the same type in Rust, but due to the limitation of procedural macros, we can not create multiple Object implementations for the same type. For example, the following code will fail to compile.

```rust,ignore,does_not_compile
#[Object]
impl Query {
    async fn users(&self) -> Vec<User> {
        todo!()
    }
}

#[Object]
impl Query {
    async fn movies(&self) -> Vec<Movie> {
        todo!()
    }
}
```

Instead, the `#[derive(MergedObject)]` macro allows you to split an object's resolvers across multiple modules or files by merging 2 or more `#[Object]` implementations into one.

**Tip:** Every `#[Object]` needs a unique name, even in a `MergedObject`, so make sure to give each object you're merging its own name.

**Note:** This works for queries and mutations. For subscriptions, see "Merging Subscriptions" below.

```rust
# extern crate async_graphql;
# use async_graphql::*;
# #[derive(SimpleObject)]
# struct User { a: i32 }
# #[derive(SimpleObject)]
# struct Movie { a: i32 }
#[derive(Default)]
struct UserQuery;

#[Object]
impl UserQuery {
    async fn users(&self) -> Vec<User> {
        todo!()
    }
}

#[derive(Default)]
struct MovieQuery;

#[Object]
impl MovieQuery {
    async fn movies(&self) -> Vec<Movie> {
        todo!()
    }
}

#[derive(MergedObject, Default)]
struct Query(UserQuery, MovieQuery);

let schema = Schema::new(
    Query::default(),
    EmptyMutation,
    EmptySubscription
);
```

> ⚠️ **MergedObject cannot be used in Interface.**

# Merging Subscriptions

Along with `MergedObject`, you can derive `MergedSubscription` or use `#[MergedSubscription]` to merge separate `#[Subscription]` blocks.

Like merging Objects, each subscription block requires a unique name.

Example:

```rust
# extern crate async_graphql;
# use async_graphql::*;
# use futures_util::stream::{Stream};
# #[derive(Default,SimpleObject)]
# struct Query { a: i32 }
#[derive(Default)]
struct Subscription1;

#[Subscription]
impl Subscription1 {
    async fn events1(&self) -> impl Stream<Item = i32> {
        futures_util::stream::iter(0..10)
    }
}

#[derive(Default)]
struct Subscription2;

#[Subscription]
impl Subscription2 {
    async fn events2(&self) -> impl Stream<Item = i32> {
        futures_util::stream::iter(10..20)
    }
}

#[derive(MergedSubscription, Default)]
struct Subscription(Subscription1, Subscription2);

let schema = Schema::new(
    Query::default(),
    EmptyMutation,
    Subscription::default()
);
```
