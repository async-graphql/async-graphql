# 定义类型的两种方式

我想你已经发现，定义一个GraphqlQL类型可以通过属性宏或者派生。

下面是一个对应表：

|类型|属性宏|派生|
|---|-----|----|
|枚举(Enum)|Enum|GQLEnum|
|简单对象(Simple Object)|SimpleObject|GQLSimpleObject|
|输入对象(Input Object)|InputObject|GQLInputObject|
|接口(Interface)|Interface|GQLInterface|
|联合(Union)|Union|GQLUnion|
|合并对象(Merged Object)|MergedObject|GQLMergedObject|
|合并订阅(Merged Subscription)|MergedSubscription|GQLMergedSubscription|

属性宏的好处在于你可以同时提供一些参数，例如：

```rust
#[SimpleObject(name = "ABC")]
struct MyObject {
    value: i32,
}
```

但是它不支持条件编译，例如：

```rust
#[SimpleObject]
struct MyObject {
    #[cfg(windows)]
    value: i32,

    #[cfg(not(windows))]
    value: i32,
}
```

派生可以支持条件编译，但它需要单独提供参数，例如：

```rust
#[derive(SimpleObject)]
#[graphql(name = "ABC")]
struct MyObject {
    value: i32,
}
```

用哪种方式来定义类型取决于你，我更加推荐使用派生。