# 默认值

你可以为输入值类型定义默认值，下面展示了在不同类型上默认值的定义方法。

## 对象字段参数

```rust
# extern crate async_graphql;
use async_graphql::*;

struct Query;

fn my_default() -> i32 {
    30
}

#[Object]
impl Query {
    // value 参数的默认值为 0，它会调用 i32::default()
    async fn test1(&self, #[graphql(default)] value: i32) -> i32 { todo!() }

    // value 参数的默认值为 10
    async fn test2(&self, #[graphql(default = 10)] value: i32) -> i32 { todo!() }

    // value 参数的默认值使用 my_default 函数的返回结果，值为 30
    async fn test3(&self, #[graphql(default_with = "my_default()")] value: i32) -> i32 { todo!() }
}
```

## 接口字段参数

```rust
# extern crate async_graphql;
# fn my_default() -> i32 { 5 }
# struct MyObj;
# #[Object]
# impl MyObj {
#    async fn test1(&self, value: i32) -> i32 { todo!() }
#    async fn test2(&self, value: i32) -> i32 { todo!() }
#    async fn test3(&self, value: i32) -> i32 { todo!() }
# }
use async_graphql::*;

#[derive(Interface)]
#[graphql(
    field(name = "test1", ty = "i32", arg(name = "value", ty = "i32", default)),
    field(name = "test2", ty = "i32", arg(name = "value", ty = "i32", default = 10)),
    field(name = "test3", ty = "i32", arg(name = "value", ty = "i32", default_with = "my_default()")),
)]
enum MyInterface {
    MyObj(MyObj),
}
```

## 输入对象 (InputObject)

```rust
# extern crate async_graphql;
# fn my_default() -> i32 { 5 }
use async_graphql::*;

#[derive(InputObject)]
struct MyInputObject {
    #[graphql(default)]
    value1: i32,

    #[graphql(default = 10)]
    value2: i32,

    #[graphql(default_with = "my_default()")]
    value3: i32,
}
```
