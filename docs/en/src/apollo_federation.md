# Apollo Federation

`Apollo Federation` is a `GraphQL` API gateway which can combine multiple GraphQL services, allowing each service to implement the subset of the API it is responsible for. You can read more in the [official documentation](https://www.apollographql.com/docs/apollo-server/federation/introduction).

`Async-graphql` supports all the functionality of `Apollo Federation`, but some modifications to your `Schema` are required.

- You can use the `extends` property declaration on `async_graphql::Object` and `async_graphql::Interface` to extend a type offered by another implementing service.

- The `external` property declares that a field comes from another service。

- The `provides` property indicates the fields provided by a service. 

## Entity lookup function

```rust
struct Query;

#[Object]
impl Query {
    #[entity]
    async fn find_user_by_id(&self, id: ID) -> User {
        User { ... }
    }

    #[entity]
    async fn find_user_by_id_with_username(&self, #[arg(key)] id: ID, username: String) -> User {
        User { ... }
    }

    #[entity]
    async fn find_user_by_id_and_username(&self, id: ID, username: String) -> User {
        User { ... }
    }
}
```

**Notice the difference between these three lookup functions, which are all looking for the `User` object.**

- `find_user_by_id`

    Use `id` to find an `User` object, the key for `User` is `id`.

- `find_user_by_id_with_username`

    Use `id` to find an `User` object, the key for `User` is `id`, and the `username` field value of the `User` object is requested.

- `find_user_by_id_and_username`

    Use `id` and `username` to find an `User` object, the keys for `User` are `id` and `username`.

For a complete example, refer to: <https://github.com/async-graphql/examples/tree/master/federation>.
