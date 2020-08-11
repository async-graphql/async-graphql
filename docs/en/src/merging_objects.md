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

Instead, the `#[derive(GQLMergedObject)]` macro allows you to split an object's resolvers across multiple file by merging 2 or more `#[Object]` implementations into one.

**Tip:** Every `#[Object]` needs a unique name even in a GQLMergedObject so make sure to give each object your merging it's own name.

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
