# Default value

You can define default values for input value types.
Below are some examples.

## Object field

```rust
use async_graphql::*;

struct Query;

fn my_default() -> i32 {
    30
}

#[Object]
impl Query {
    // The default value of the value parameter is 0, it will call i32::default()
    fn test1(&self, #[graphql(default)] value: i32) {}

    // The default value of the value parameter is 10
    fn test2(&self, #[graphql(default = 10)] value: i32) {}

    // The default value of the value parameter uses the return result of the my_default function, the value is 30.
    fn test3(&self, #[graphql(default_with = "my_default()")] value: i32) {}
}
```

## Interface field

```rust
use async_graphql::*;

#[derive(Interface)]
#[graphql(
    field(name = "test1", arg(name = "value", default)),
    field(name = "test2", arg(name = "value", default = 10)),
    field(name = "test3", arg(name = "value", default_with = "my_default()")),
)]
enum MyInterface {
    MyObj(MyObj),
}
```

## Input object field

```rust
use async_graphql::*;

#derive(InputObject)
struct MyInputObject {
    #[graphql(default)]
    value1: i32,

    #[graphql(default = 10)]
    value2: i32,

    #[graphql(default_with = "my_default()")]
    value3: i32,
}
```
