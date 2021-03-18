# SimpleObject

`SimpleObject` directly maps all the fields of a struct to GraphQL object. You cannot define a resolver function on it - for that, see [Object](define_complex_object.html).

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

## Complex resolvers

Sometimes most of the fields of a GraphQL object simply return the value of the structure member, but a few
fields are calculated. Usually we use the `Object` macro to define such a GraphQL object.

But this can be done more beautifully with the `ComplexObject` macro. We can use the `SimpleObject` macro to define
some simple fields, and use the `ComplexObject` macro to define some other fields that need to be calculated.

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