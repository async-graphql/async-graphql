# Apollo Federation

`Apollo Federation` is a `GraphQL` API gateway which can combine multiple GraphQL services, allowing each service to implement the subset of the API it is responsible for. You can read more in the [official documentation](https://www.apollographql.com/docs/apollo-server/federation/introduction)。

`Async-GraphQL` supports all the functionality of `Apollo Federation`, but some modifications to your `Schema` are required.

- You can use the `extends` property declaration on `async_graphql::Object` and `async_graphql::Interface` to extend a type offered by another implementing service.

- The `external` property declares that a field comes from another service。

- The `provides` property indicates the fields provided by a service. 

The definition of a root Query type is slightly different. An entity search function must be defined. For example:

```rust
struct Query;

#[Object]
impl Query {
    #[entity]
    async fn find_user_by_id(&self, id: ID) -> User {
        User { id }
    }
}
```

This is equivalent to:

```graphql
type User @key(id: ID!) {
    id: ID!,
}
```

For a complete example, refer to: https://github.com/async-graphql/examples/tree/master/federation
