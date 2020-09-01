# Merging Objects

Usually we can create multiple implementations for the same type in Rust, but due to the limitation of procedural macros, we can not create multiple Object implementations for the same type. For example, the following code will fail to compile.

```rust
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

Instead, the `#[derive(GQLMergedObject)]`/`#[MergedObject]` macro allows you to split an object's resolvers across multiple modules or files by merging 2 or more `#[Object]` implementations into one.

**Tip:** Every `#[Object]` needs a unique name, even in a `GQLMergedObject`, so make sure to give each object you're merging its own name.

**Note:** This works for queries and mutations. For subscriptions, see "Merging Subscriptions" below.

```rust
#[Object]
impl UserQuery {
    async fn users(&self) -> Vec<User> {
        todo!()
    }
}

#[Object]
impl MovieQuery {
    async fn movies(&self) -> Vec<Movie> {
        todo!()
    }
}

#[derive(GQLMergedObject, Default)]
struct Query(UserQuery, MovieQuery);

let schema = Schema::new(
    Query::default(),
    EmptyMutation,
    EmptySubscription
);
```

# Merging Subscriptions

Along with `GQLMergedObject`, you can derive `GQLMergedSubscription` or use `#[MergedSubscription]` to merge separate `#[Subscription]` blocks.

Like merging Objects, each subscription block requires a unique name.

Example:

```rust
#[derive(Default)]
struct Subscription1;

#[Subscription]
impl Subscription1 {
    async fn events1(&self) -> impl Stream<Item = i32> {
        futures::stream::iter(0..10)
    }
}

#[derive(Default)]
struct Subscription2;

#[Subscription]
impl Subscription2 {
    async fn events2(&self) -> impl Stream<Item = i32> {
        futures::stream::iter(10..20)
    }
}

#[derive(GQLMergedSubscription, Default)]
struct Subscription(Subscription1, Subscription2);

let schema = Schema::new(
    Query::default(),
    EmptyMutation,
    Subscription::default()
);
```
