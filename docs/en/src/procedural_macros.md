# Two ways to define types

I think you have discovered that GraphQL types can be defined using both an attribute macro and a derive.

The following is the corresponding table:

|Type|Attribute macro|Derive|
|---|-----|----|
|Enum|Enum|GQLEnum|
|Simple Object|SimpleObject|GQLSimpleObject|
|Input Object|InputObject|GQLInputObject|
|Interface|Interface|GQLInterface|
|Union|Union|GQLUnion|
|Merged Object|MergedObject|GQLMergedObject|
|Merged Subscription|MergedSubscription|GQLMergedSubscription|

The advantage of the attribute macro is that you can provide parameters at the same time, for example：

```rust
#[SimpleObject(name = "ABC")]
struct MyObject {
    value: i32,
}
```

**However, attribute macros do not support conditional compilation**. The following does not work:

```rust
#[SimpleObject]
struct MyObject {
    #[cfg(windows)]
    value: i32,

    #[cfg(not(windows))]
    value: i32,
}
```

Deriving, on the other hand, does support conditional compilation, but as derive macros can't take parameters you need to provide them separately. For example:

```rust
#[derive(SimpleObject)]
#[graphql(name = "ABC")]
struct MyObject {
    value: i32,
}
```

_Which way you use to define types is up to you, personally I prefer to use derive._
