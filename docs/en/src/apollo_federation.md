# Apollo Federation

`Apollo Federation` is a `GraphQL` API gateway which can combine multiple GraphQL services, allowing each service to implement the subset of the API it is responsible for. You can read more in the [official documentation](https://www.apollographql.com/docs/apollo-server/federation/introduction).

`Async-graphql` supports all the functionality of `Apollo Federation`, but some modifications to your `Schema` are required.

- You can use the `extends` property declaration on `async_graphql::Object` and `async_graphql::Interface` to extend a type offered by another implementing service.

- The `external` property declares that a field comes from another service。

- The `provides` directive is used to annotate the expected returned fieldset from a field on a base type that is guaranteed to be selectable by the gateway. 

- The `requires` directive is used to annotate the required input fieldset from a base type for a resolver. It is used to develop a query plan where the required fields may not be needed by the client, but the service may need additional information from other services.

## Entity lookup function

```rust
# extern crate async_graphql;
# use async_graphql::*;
# #[derive(SimpleObject)]
# struct User { id: ID }
struct Query;

#[Object]
impl Query {
    #[graphql(entity)]
    async fn find_user_by_id(&self, id: ID) -> User {
        User { id }
    }

    #[graphql(entity)]
    async fn find_user_by_id_with_username(&self, #[graphql(key)] id: ID, username: String) -> User {
        User { id }
    }

    #[graphql(entity)]
    async fn find_user_by_id_and_username(&self, id: ID, username: String) -> User {
        User { id }
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

## Defining a compound primary key

A single primary key can consist of multiple fields, and even nested fields, you can use `InputObject` to implements a nested primary key.

In the following example, the primary key of the `User` object is `key { a b }`.

```rust
# extern crate async_graphql;
# use async_graphql::*;
# #[derive(SimpleObject)]
# struct User { id: i32 }
#[derive(InputObject)]
struct NestedKey {
  a: i32,
  b: i32,
}

struct Query;

#[Object]
impl Query {
  #[graphql(entity)]
  async fn find_user_by_key(&self, key: NestedKey) -> User {
    User { id: key.a }
  }
}
```
