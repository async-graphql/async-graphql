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
    fn test1(&self, #[arg(default)] value: i32) {}
    
    // The default value of the value parameter is 10
    fn test2(&self, #[arg(default = 10)] value: i32) {}

    // The default value of the value parameter uses the return result of the my_default function, the value is 30.
    fn test3(&self, #[arg(default_with = "my_default()")] value: i32) {}
}
```

## Interface field

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

## Input object field

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
