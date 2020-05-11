# Query and Mutation

## Query root object

The query root object is a GraphQL object with a definition similar to other objects. Resolve functions for all fields of the query object are executed concurrently.


```rust
use async_graphql::*;

struct Query;

#[GqlObject]
impl Query {
    async fn user(&self, username: String) -> GqlFieldResult<Option<User>> {
        // Look up users from the database
    }
}

```

## Mutation root object

The mutation root object is also a GraphQL object, but it executes sequentially. One mutation following from another will only be executed only after the first mutation is completed.

The following mutation root object provides an example of user registration and login:

```rust
use async_graphql::*;

struct Mutation;

#[GqlObject]
impl Mutation {
    async fn signup(&self, username: String, password: String) -> Result<bool> {
        // User signup
    }

    async fn login(&self, username: String, password: String) -> Result<String> {
        // User login (generate token)
    }
}
```

