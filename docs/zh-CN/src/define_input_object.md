# 输入对象 (InputObject)

你可以定义一个对象作为参数类型，GraphQL 称之为`Input Object`，输入对象的定义方式和[简单对象](define_simple_object.md)很像，不同的是，简单对象只能用于输出，而输入对象只能用于输入。

你也通过可选的`#[graphql]`属性来给字段添加描述，重命名。

```rust
# extern crate async_graphql;
# #[derive(SimpleObject)]
# struct User { a: i32 }
use async_graphql::*;

#[derive(InputObject)]
struct Coordinate {
    latitude: f64,
    longitude: f64,
}

struct Mutation;

#[Object]
impl Mutation {
    async fn users_at_location(&self, coordinate: Coordinate, radius: f64) -> Vec<User> {
        // 将坐标写入数据库
        // ...
#       todo!()
    }
}
```

## 泛型

如果你希望其它类型能够重用`InputObject`，则可以定义泛型的`InputObject`，并指定具体的类型。

在下面的示例中，创建了两种`InputObject`类型：

```rust
# extern crate async_graphql;
# use async_graphql::*;
# #[derive(InputObject)]
# struct SomeType { a: i32 }
# #[derive(InputObject)]
# struct SomeOtherType { a: i32 }
#[derive(InputObject)]
#[graphql(concrete(name = "SomeName", params(SomeType)))]
#[graphql(concrete(name = "SomeOtherName", params(SomeOtherType)))]
pub struct SomeGenericInput<T: InputType> {
    field1: Option<T>,
    field2: String
}
```

注意：每个泛型参数必须实现`InputType`，如上所示。

生成的 SDL 如下：

```gql
input SomeName {
  field1: SomeType
  field2: String!
}

input SomeOtherName {
  field1: SomeOtherType
  field2: String!
}
```

在其它`InputObject`中使用具体的泛型类型：

```rust
# extern crate async_graphql;
# use async_graphql::*;
# #[derive(InputObject)]
# struct SomeType { a: i32 }
# #[derive(InputObject)]
# struct SomeOtherType { a: i32 }
# #[derive(InputObject)]
# #[graphql(concrete(name = "SomeName", params(SomeType)))]
# #[graphql(concrete(name = "SomeOtherName", params(SomeOtherType)))]
# pub struct SomeGenericInput<T: InputType> {
#     field1: Option<T>,
#     field2: String
# }
#[derive(InputObject)]
pub struct YetAnotherInput {
    a: SomeGenericInput<SomeType>,
    b: SomeGenericInput<SomeOtherType>,
}
```

你可以将多个通用类型传递给`params（）`，并用逗号分隔。
