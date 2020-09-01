# SimpleObject

`SimpleObject` directly maps all the fields of a struct to GraphQL object. You cannot define a resolver function on it - for that, see [Object](define_complex_object.html).

The example below defines an object `MyObject` which includes the fields `a` and `b`. `c` will be not mapped to GraphQL as it is labelled as `#[field(skip)]`

```rust
use async_graphql::*;

#[SimpleObject]
struct MyObject {
    a: i32,
    b: i32,

    #[field(skip)]
    c: i32,
}
```
