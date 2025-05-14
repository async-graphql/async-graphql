# SimpleObject

`SimpleObject` directly maps all the fields of a struct to GraphQL object.
If you don't require automatic mapping of fields, see [Object](define_complex_object.html).

The example below defines an object `MyObject` which includes the fields `a` and `b`. `c` will be not mapped to GraphQL as it is labelled as `#[graphql(skip)]`

```rust
# extern crate async_graphql;
use async_graphql::*;

#[derive(SimpleObject)]
struct MyObject {
    /// Value a
    a: i32,

    /// Value b
    b: i32,

    #[graphql(skip)]
    c: i32,
}
```

## User-defined resolvers

Sometimes most of the fields of a GraphQL object simply return the value of the structure member, but a few
fields are calculated. In this case, the [Object](define_complex_object.html) macro cannot be used unless you hand-write all the resolvers.

The `ComplexObject` macro works in conjunction with the `SimpleObject` macro. The `SimpleObject` derive macro defines
the non-calculated fields, where as the `ComplexObject` macro let's you write user-defined resolvers for the calculated fields.

Resolvers added to `ComplexObject` adhere to the same rules as resolvers of [Object](define_complex_object.html).

```rust
# extern crate async_graphql;
# use async_graphql::*;
#[derive(SimpleObject)]
#[graphql(complex)] // NOTE: If you want the `ComplexObject` macro to take effect, this `complex` attribute is required.
struct MyObj {
    a: i32,
    b: i32,
}

#[ComplexObject]
impl MyObj {
    async fn c(&self) -> i32 {
        self.a + self.b
    }
}
```

## Used for both input and output

```rust
# extern crate async_graphql;
# use async_graphql::*;
#[derive(SimpleObject, InputObject)]
#[graphql(input_name = "MyObjInput")] // Note: You must use the input_name attribute to define a new name for the input type, otherwise a runtime error will occur.
struct MyObj {
    a: i32,
    b: i32,
}
```

## Flatten fields

You can flatten fields by adding `#[graphql(flatten)]`, i.e.:

```rust
# extern crate async_graphql;
# use async_graphql::*;
#[derive(SimpleObject)]
pub struct ChildObject {
    b: String,
    c: String,
}

#[derive(SimpleObject)]
pub struct ParentObject {
    a: String,
    #[graphql(flatten)]
    child: ChildObject,
}

// Is the same as

#[derive(SimpleObject)]
pub struct Object {
    a: String,
    b: String,
    c: String,
}
```
