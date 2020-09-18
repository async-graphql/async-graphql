# Cache control

Production environments often rely on caching to improve performance.

A GraphQL query will call multiple resolver functions and each resolver can have a different cache definition. Some may cache for a few seconds, some may cache for a few hours, some may be the same for all users, and some may be different for each session.

`Async-graphql` provides a mechanism that allows you to define the cache time and scope for each resolver.

You can define cache parameters on the object or on its fields. The following example shows two uses of cache control parameters.

You can use `max_age` parameters to control the age of the cache (in seconds), and you can also use `public` and `private` to control the scope of the cache. When you do not specify it, the scope will default to `public`.

when querying multiple resolvers, the results of all cache control parameters will be combined and the `max_age` minimum value will be taken. If the scope of any object or field is `private`, the result will be `private`.

We can use `QueryResponse` to get a merged cache control result from a query result, and call `CacheControl::value` to get the corresponding HTTP header.

```rust
#[Object(cache_control(max_age = 60))]
impl Query {
    #[field(cache_control(max_age = 30))]
    async fn value1(&self) -> i32 {
    }

    #[field(cache_control(private))]
    async fn value2(&self) -> i32 {
    }

    async fn value3(&self) -> i32 {
    }
}
```

The following are different queries corresponding to different cache control results:

```graphql
# max_age=30
{ value1 }
```

```graphql
# max_age=30, private
{ value1 value2 }
```

```graphql
# max_age=60
{ value3 }
```

