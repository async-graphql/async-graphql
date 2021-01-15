# InputObject

You can use an `Object` as an argument, and GraphQL calls it an `InputObject`.

The definition of `InputObject` is similar to [SimpleObject](define_simple_object.md), but
`SimpleObject` can only be used as output and `InputObject` can only be used as input.

You can add optional `#[graphql]` attributes to add descriptions or rename the field.

```rust
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
    }
}
```

## Generic `InputObject`s

If you want to reuse an `InputObject` for other types, you can define a generic InputObject
and specify how its concrete types should be implemented.

In the following example, two `InputObject` types are created:

```rust
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
#[derive(InputObject)]
pub struct YetAnotherInput {
    a: SomeGenericInput<SomeType>,
    b: SomeGenericInput<SomeOtherType>,
}
```

You can pass multiple generic types to `params()`, separated by a comma.
