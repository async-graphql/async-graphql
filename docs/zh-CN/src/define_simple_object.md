# 简单对象 (SimpleObject)

简单对象是把 Rust 结构的所有字段都直接映射到 GraphQL 对象，不支持定义单独的 Resolver 函数。

下面的例子定义了一个名称为 MyObject 的对象，包含字段`a`和`b`，`c`由于标记为`#[graphql(skip)]`，所以不会映射到 GraphQL。

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

## 泛型

如果你希望其它类型能够重用`SimpleObject`，则可以定义泛型的`SimpleObject`，并指定具体的类型。

在下面的示例中，创建了两种`SimpleObject`类型：

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

注意：每个泛型参数必须实现`OutputType`，如上所示。

生成的 SDL 如下：

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

你可以将多个通用类型传递给`params（）`，并用逗号分隔。

## 复杂字段

有时 GraphQL 对象的大多数字段仅返回结构成员的值，但是少数字段需要计算。通常我们使用`Object`宏来定义这样一个 GraphQL 对象。

用`ComplexObject`宏可以更漂亮的完成这件事，我们可以使用`SimpleObject`宏来定义
一些简单的字段，并使用`ComplexObject`宏来定义其他一些需要计算的字段。

```rust
# extern crate async_graphql;
# use async_graphql::*;
#[derive(SimpleObject)]
#[graphql(complex)] // 注意：如果你希望 ComplexObject 宏生效，complex 属性是必须的
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

## 同时用于输入和输出

```rust
# extern crate async_graphql;
# use async_graphql::*;
#[derive(SimpleObject, InputObject)]
#[graphql(input_name = "MyObjInput")] // 注意：你必须用 input_name 属性为输入类型定义一个新的名称，否则将产生一个运行时错误。
struct MyObj {
    a: i32,
    b: i32,
}
```
