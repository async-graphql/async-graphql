# 简单对象(SimpleObject)

简单对象是把Rust结构的所有字段都直接映射到GraphQL对象，不支持定义单独的Resolver函数。

下面的例子定义了一个名称为MyObject的对象，包含字段`a`和`b`，`c`由于标记为`#[graphql(skip)]`，所以不会映射到GraphQL。

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

## 泛型

如果你希望其它类型能够重用`SimpleObject`，则可以定义泛型的`SimpleObject`，并指定具体的类型。

在下面的示例中，创建了两种`SimpleObject`类型：

```rust
#[derive(SimpleObject)]
#[graphql(concrete(name = "SomeName", params(SomeType)))]
#[graphql(concrete(name = "SomeOtherName", params(SomeOtherType)))]
pub struct SomeGenericObject<T: OutputType> {
    field1: Option<T>,
    field2: String
}
```

注意：每个泛型参数必须实现`OutputType`，如上所示。

生成的SDL如下:

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

在其它`Object`中使用具体的泛型类型：

```rust
#[derive(SimpleObject)]
pub struct YetAnotherObject {
    a: SomeGenericObject<SomeType>,
    b: SomeGenericObject<SomeOtherType>,
}
```

你可以将多个通用类型传递给`params（）`，并用逗号分隔。
