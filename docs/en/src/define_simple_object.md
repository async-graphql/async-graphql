# SimpleObject

`SimpleObject` directly map all field of a struct to GraphQL object, you cannot define a Resolve function on it.

The example below defined an object `MyObject`, including field `a` and `b`. `c` will be not mapped to GraphQL as it is labelled as `#[field(skip)]`

```rust
use async_graphql::*;

#[GqlSimpleObject]
struct MyObject {
    a: i32,
    b: i32,

    #[field(skip)]
    c: i32,
}
```
