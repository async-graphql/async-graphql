# Two ways to define types

I think you have discovered that defining a GraphqlQL type can be use attribute macro or derive.

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

The advantage of attribute macro is that you can provide some parameters at the same time, for example：

```rust
#[SimpleObject(name = "ABC")]
struct MyObject {
    value: i32,
}
```

**But it does not support conditional compilation**, for example:

```rust
#[SimpleObject]
struct MyObject {
    #[cfg(windows)]
    value: i32,

    #[cfg(not(windows))]
    value: i32,
}
```

**Derive can support conditional compilation**, but it needs to provide parameters separately, for example:

```rust
#[derive(SimpleObject)]
#[graphql(name = "ABC")]
struct MyObject {
    value: i32,
}
```

_Which way to define the type is up to you, I prefer to use derive._