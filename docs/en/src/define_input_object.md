# InputObject

You can use an `Object` as an argument, and GraphQL calls it an `InputObject`.

The definition of `InputObject` is similar to [SimpleObject](define_simple_object.md), but
`SimpleObject` can only be used as output and `InputObject` can only be used as input.

You can add optional `#[graphql]` attributes to add descriptions or rename the field.

```rust
# extern crate async_graphql;
# #[derive(SimpleObject)]
# struct User { a: i32 }
use async_graphql::*;

#[derive(InputObject)]
struct Coordinate {
    latitude: f64,
    longitude: f64
}

struct Mutation;

#[Object]
impl Mutation {
    async fn users_at_location(&self, coordinate: Coordinate, radius: f64) -> Vec<User> {
        // Writes coordination to database.
        // ...
#       todo!()
    }
}
```

## Generic `InputObject`s

If you want to reuse an `InputObject` for other types, you can define a generic InputObject
and specify how its concrete types should be implemented.

In the following example, two `InputObject` types are created:

```rust
# extern crate async_graphql;
# use async_graphql::*;
# #[derive(InputObject)]
# struct SomeType { a: i32 }
# #[derive(InputObject)]
# struct SomeOtherType { a: i32 }
#[derive(InputObject)]
#[graphql(concrete(name = "SomeName", params(SomeType)))]
#[graphql(concrete(name = "SomeOtherName", params(SomeOtherType)))]
pub struct SomeGenericInput<T: InputType> {
    field1: Option<T>,
    field2: String
}
```

Note: Each generic parameter must implement `InputType`, as shown above.

The schema generated is:

```gql
input SomeName {
  field1: SomeType
  field2: String!
}

input SomeOtherName {
  field1: SomeOtherType
  field2: String!
}
```

In your resolver method or field of another input object, use as a normal generic type:

```rust
# extern crate async_graphql;
# use async_graphql::*;
# #[derive(InputObject)]
# struct SomeType { a: i32 }
# #[derive(InputObject)]
# struct SomeOtherType { a: i32 }
# #[derive(InputObject)]
# #[graphql(concrete(name = "SomeName", params(SomeType)))]
# #[graphql(concrete(name = "SomeOtherName", params(SomeOtherType)))]
# pub struct SomeGenericInput<T: InputType> {
#     field1: Option<T>,
#     field2: String
# }
#[derive(InputObject)]
pub struct YetAnotherInput {
    a: SomeGenericInput<SomeType>,
    b: SomeGenericInput<SomeOtherType>,
}
```

You can pass multiple generic types to `params()`, separated by a comma.

## Redacting sensitive data

If any part of your input is considered sensitive and you wish to redact it, you can mark it with `secret` directive. For example:

```rust
# extern crate async_graphql;
# use async_graphql::*;
#[derive(InputObject)]
pub struct CredentialsInput {
    username: String,
    #[graphql(secret)]
    password: String,
}
```

## Flattening fields

You can add `#[graphql(flatten)]` to a field to inline keys from the field type into it's parent. For example:

```rust
# extern crate async_graphql;
# use async_graphql::*;
#[derive(InputObject)]
pub struct ChildInput {
    b: String,
    c: String,
}

#[derive(InputObject)]
pub struct ParentInput {
    a: String,
    #[graphql(flatten)]
    child: ChildInput,
}

// Is the same as

#[derive(InputObject)]
pub struct Input {
    a: String,
    b: String,
    c: String,
}
```
