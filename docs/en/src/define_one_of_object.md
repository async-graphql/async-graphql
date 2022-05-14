# OneofObject

A `OneofObject` is a special type of `InputObject`, in which only one of its fields must be set and is not-null.
It is especially useful when you want a user to be able to choose between several potential input types.

This feature is still an [RFC](https://github.com/graphql/graphql-spec/pull/825) and therefore not yet officially part of the GraphQL spec, but `Async-graphql` already supports it!

```rust
use async_graphql::*;

#[derive(OneofObject)]
enum UserBy {
    Email(String),
    RegistrationNumber(i64),
    Address(Address)
}

#[derive(InputObject)]
struct Address {
    street: String,
    house_number: String,
    city: String,
    zip: String,
}

struct Query {}

#[Object]
impl Query {
    async fn search_users(&self, by: Vec<UserBy>) -> Vec<User> {
        // ... Searches and returns a list of users ...
    }
}
```

As you can see, a `OneofObject` is represented by an `enum` in which each variant contains another `InputType`. This means that you can use [`InputObject`](define_input_object.md) as variant too.
