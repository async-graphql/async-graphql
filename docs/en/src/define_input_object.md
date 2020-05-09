# InputObject

<!--Input Object and SimpleObject inconsistant space.-->
You can define an `Object` as argument, GraphQL calls it `InputObject`.
The definition of `InputObject` is similar to [SimpleObject](define_simple_object.md).
However, `InputObject` can only be used for output and `SimpleObject` can only be used as input.

`InputObject` don't need a `#[field]` for each field, every field is `InputValue`.
But you can add optional `#[field]` to add description or rename the field.

```rust
use async_graphql::*;

#[InputObject]
struct Coordinate {
    latitude: f64,

    #[field(desc = "...")]
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