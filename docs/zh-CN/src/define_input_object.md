# 输入对象(InputObject)

你可以定义一个对象作为参数类型，GraphQL称之为`Input Object`，输入对象的定义方式和[简单对象](define_simple_object.md)很像，不同的是，简单对象只能用于输出，而输入对象只能用于输入。

你也通过可选的`#[graphql]`属性来给字段添加描述，重命名。

```rust
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
    }
}
```