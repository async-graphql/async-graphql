# Query and Mutation

## Query root object

The query root object is a GraphQL object with a definition similar to other objects. Resolver functions for all fields of the query object are executed concurrently.

```rust
# extern crate async_graphql;
use async_graphql::*;
# #[derive(SimpleObject)]
# struct User { a: i32 }

struct Query;

#[Object]
impl Query {
    async fn user(&self, username: String) -> Result<Option<User>> {
        // Look up users from the database
#        todo!()
    }
}

```

## Mutation root object

The mutation root object is also a GraphQL object, but it executes sequentially. One mutation following from another will only be executed only after the first mutation is completed.

The following mutation root object provides an example of user registration and login:

```rust
# extern crate async_graphql;
use async_graphql::*;

struct Mutation;

#[Object]
impl Mutation {
    async fn signup(&self, username: String, password: String) -> Result<bool> {
        // User signup
#        todo!()
    }

    async fn login(&self, username: String, password: String) -> Result<String> {
        // User login (generate token)
#        todo!()
    }
}
```
