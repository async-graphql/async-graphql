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

## Using the Same Struct for `InputObject`s and `SimpleObject`s

It is not currently possible to derive `InputObject` and `SimpleObject` on the same struct ( see [#149] ). In some cases, though, it may be desireable to use the same Rust struct to represent both the input and output types. In this case you can use the [`Json`] struct as a wrapper for the input type, and then derive `SimpleObject` on the output type like normal. For instance:

```rust
#[derive(SimpleObject)]
struct Coordinate {
    latitude: f64,
    longitude: f64
}

struct Mutation;

#[Object]
impl Mutation {
    async fn add_coordinate(&self, coordinate: Json<Coordinate>) -> Coordinate {
        // Writes coordination to database.
        // ...
        
        // Returns coordinate
    }
}
```

The disadvantage of this technique is that the input will not be statically typed in the GraphQL schema. The input type of the mutation in this instance would simply show the type `JSON` instead of outlining all of the fields that are accepted by the input. Note that errors _will_ still be reported when running the mutation if the input does not match the Rust structure.

This limitation should be removed in a future release making this workaround unnecessary.

[`Json`]: https://docs.rs/async-graphql/latest/async_graphql/types/struct.Json.html
[#149]: https://github.com/async-graphql/async-graphql/issues/149

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
