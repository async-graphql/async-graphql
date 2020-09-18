# SimpleObject

`SimpleObject` directly maps all the fields of a struct to GraphQL object. You cannot define a resolver function on it - for that, see [Object](define_complex_object.html).

The example below defines an object `MyObject` which includes the fields `a` and `b`. `c` will be not mapped to GraphQL as it is labelled as `#[field(skip)]`

```rust
use async_graphql::*;

#[derive(SimpleObject)]
struct MyObject {
    /// Value a
    a: i32,
    
    #[field(desc = "Value b")]
    b: i32,

    #[field(skip)]
    c: i32,
}
```
