# SimpleObject

`SimpleObject` directly maps all the fields of a struct to GraphQL object.
If you don't require automatic mapping of fields, see [Object](define_complex_object.html).

The example below defines an object `MyObject` which includes the fields `a` and `b`. `c` will be not mapped to GraphQL as it is labelled as `#[graphql(skip)]`

```rust
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

The `ComplexObject` macro works in conjuction with the `SimpleObject` macro. The `SimpleObject` derive macro defines
the non-calculated fields, where as the `ComplexObject` macro let's you write user-defined resolvers for the calculated fields.

Resolvers added to `ComplexObject` adhere to the same rules as resolvers of [Object](define_complex_object.html).

```rust
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

## Generic `SimpleObject`s

If you want to reuse an `SimpleObject` for other types, you can define a generic SimpleObject
and specify how its concrete types should be implemented.

In the following example, two `SimpleObject` types are created:

```rust
#[derive(SimpleObject)]
#[graphql(concrete(name = "SomeName", params(SomeType)))]
#[graphql(concrete(name = "SomeOtherName", params(SomeOtherType)))]
pub struct SomeGenericObject<T: OutputType> {
    field1: Option<T>,
    field2: String
}
```

Note: Each generic parameter must implement `OutputType`, as shown above.

The schema generated is:

```gql
type SomeName {
  field1: SomeType
  field2: String!
}

type SomeOtherName {
  field1: SomeOtherType
  field2: String!
}
```

In your resolver method or field of another object, use as a normal generic type:

```rust
#[derive(SimpleObject)]
pub struct YetAnotherObject {
    a: SomeGenericObject<SomeType>,
    b: SomeGenericObject<SomeOtherType>,
}
```

You can pass multiple generic types to `params()`, separated by a comma.
