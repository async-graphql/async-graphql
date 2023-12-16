# 自定义标量

`Async-graphql`已经内置了绝大部分常用的标量类型，同时你也能自定义标量。

实现`Async-graphql::Scalar`即可自定义一个标量，你只需要实现一个解析函数和输出函数。

下面的例子定义一个 64 位整数标量，但它的输入输出都是字符串。 (`Async-graphql`已经内置了对 64 位整数的支持，正是采用字符串作为输入输出)

```rust
# extern crate async_graphql;
use async_graphql::*;


struct StringNumber(i64);

#[Scalar]
impl ScalarType for StringNumber {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(value) = &value {
            // 解析整数
            Ok(value.parse().map(StringNumber)?)
        } else {
            // 类型不匹配
            Err(InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_string())
    }
}

```

## 使用`scalar!`宏定义标量

如果你的类型实现了`serde :: Serialize`和`serde :: Deserialize`，那么可以使用此宏更简单地定义标量。

```rust
# extern crate async_graphql;
# extern crate serde;
# use async_graphql::*;
# use serde::{Serialize, Deserialize};
# use std::collections::HashMap;
#[derive(Serialize, Deserialize)]
struct MyValue {
    a: i32,
    b: HashMap<String, i32>,     
}

scalar!(MyValue);

// 重命名为 `MV`.
// scalar!(MyValue, "MV");

// 重命名为 `MV` 并且添加描述。
// scalar!(MyValue, "MV", "This is my value");
```
