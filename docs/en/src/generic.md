# Generics

It is possible to define reusable objects using generics; however
each concrete instantiation of a generic object must be given a unique GraphQL type name.
There are two ways of specifying these concrete names: concrete instantiation and the `TypeName` trait.

## Concrete Instantiation

In the following example, two `SimpleObject` types are created:

```rust
# extern crate async_graphql;
# use async_graphql::*;
# #[derive(SimpleObject)]
# struct SomeType { a: i32 }
# #[derive(SimpleObject)]
# struct SomeOtherType { a: i32 }
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
# SomeGenericObject<SomeType>
type SomeName {
  field1: SomeType
  field2: String!
}

# SomeGenericObject<SomeOtherType>
type SomeOtherName {
  field1: SomeOtherType
  field2: String!
}
```

In your resolver method or field of another object, use as a normal generic type:

```rust
# extern crate async_graphql;
# use async_graphql::*;
# #[derive(SimpleObject)]
# struct SomeType { a: i32 }
# #[derive(SimpleObject)]
# struct SomeOtherType { a: i32 }
# #[derive(SimpleObject)]
# #[graphql(concrete(name = "SomeName", params(SomeType)))]
# #[graphql(concrete(name = "SomeOtherName", params(SomeOtherType)))]
# pub struct SomeGenericObject<T: OutputType> {
#     field1: Option<T>,
#     field2: String,
# }
#[derive(SimpleObject)]
pub struct YetAnotherObject {
    a: SomeGenericObject<SomeType>,
    b: SomeGenericObject<SomeOtherType>,
}
```

You can pass multiple generic types to `params()`, separated by a comma.

## `TypeName` trait

Some type names can be derived.

```rust
# extern crate async_graphql;
# use async_graphql::*;
# use std::borrow::Cow;
#[derive(SimpleObject)]
#[graphql(name_type)] // Use `TypeName` trait
struct Bag<T: OutputType> {
    content: Vec<T>,
    len: usize,
}

impl<T: OutputType> TypeName for Bag<T> {
    fn type_name() -> Cow<'static, str> {
        format!("{}Bag", <T as OutputType>::type_name()).into()
    }
}
```

Using `bool` and `String` the generated schema is:
```gql
# Bag<bool>
type BooleanBag {
    content: [Boolean!]!
    len: Int!
}

# Bag<String>
type StringBag {
    content: [String!]!
    len: Int!
}
```