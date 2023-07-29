# Default value

You can define default values for input value types.
Below are some examples.

## Object field

```rust
# extern crate async_graphql;
use async_graphql::*;

struct Query;

fn my_default() -> i32 {
    30
}

#[Object]
impl Query {
    // The default value of the value parameter is 0, it will call i32::default()
    async fn test1(&self, #[graphql(default)] value: i32) -> i32 { todo!() }

    // The default value of the value parameter is 10
    async fn test2(&self, #[graphql(default = 10)] value: i32) -> i32 { todo!() }
    
    // The default value of the value parameter uses the return result of the my_default function, the value is 30.
    async fn test3(&self, #[graphql(default_with = "my_default()")] value: i32) -> i32 { todo!() }
}
```

## Interface field

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

## Input object field

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
