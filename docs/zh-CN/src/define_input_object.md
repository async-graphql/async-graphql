# 输入对象(InputObject)

你可以定义一个对象作为参数类型，GraphQL称之为`Input Object`，输入对象的定义方式和[简单对象](define_simple_object.md)很像，不同的是，简单对象只能用于输出，而输入对象只能用于输入。

输入对象不需要为每个字段指定`#[field]`标记，它每个字段都是`Input Value`。但你也可以通过可选的`#[field]`标记来给字段添加描述，重命名。

```rust
use async_graphql::*;

#[InputObject]
struct Coordinate {
    latitude: f64,

    #[field(desc = "...")]
    longitude: f64,
}

struct Mutation;

#[Object]
impl Mutation {
    async fn users_at_location(&self, coordinate: Coordinate, radius: f64) -> Vec<User> {
        // 将坐标写入数据库
        // ...
    }
}
```