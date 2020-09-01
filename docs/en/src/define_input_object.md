# InputObject

<!--Input Object and SimpleObject inconsistant space.-->
You can use an `Object` as an argument, and GraphQL calls it an `InputObject`.
The definition of `InputObject` is similar to [SimpleObject](define_simple_object.md), but
`SimpleObject` can only be used as output and `InputObject` can only be used as input.

`InputObject` doesn't need a `#[field]` for each field, every field is an `InputValue`.
You can add optional `#[field]` attributes to add descriptions or rename the field.

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
