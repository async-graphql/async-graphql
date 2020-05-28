# 默认值

你可以为输入值类型定义默认值，下面展示了在不同类型上默认值的定义方法。

## 对象字段参数

```rust
use async_graphql::*;

struct Query;

fn my_default() -> i32 {
    30
}

#[Object]
impl Query {
    // value参数的默认值为0，它会调用i32::default()
    fn test1(&self, #[arg(default)] value: i32) {}
    
    // value参数的默认值为10
    fn test2(&self, #[arg(default = 10)] value: i32) {}

    // value参数的默认值使用my_default函数的返回结果，值为30
    fn test3(&self, #[arg(default_with = "my_default()")] value: i32) {}
}
```

## 接口字段参数

```rust
use async_graphql::*;

#[Interface(
    field(name = "test1", arg(name = "value", default)),
    field(name = "test2", arg(name = "value", default = 10)),
    field(name = "test3", arg(name = "value", default = "my_default()")),
)]
enum MyInterface {
    MyObj(MyObj),
}
```

## 输入对象(InputObject)

```rust
use async_graphql::*;

struct MyInputObject {
    #[field(default)]
    value1: i32,
    
    #[field(default = 10)]
    value2: i32,

    #[field(default = "my_default()")]
    value3: i32,
}
```