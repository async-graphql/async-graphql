# 错误处理

Resolver 函数可以返回一个 Result 类型，以下是 Result 的定义：

```rust,ignore
type Result<T> = std::result::Result<T, Error>;
```

任何错误都能够被转换为`Error`，并且你还能扩展标准的错误信息。

下面是一个例子，解析一个输入的字符串到整数，当解析失败时返回错误，并且附加额外的错误信息。

```rust
# extern crate async_graphql;
# use std::num::ParseIntError;
use async_graphql::*;

struct Query;

#[Object]
impl Query {
    async fn parse_with_extensions(&self, input: String) -> Result<i32> {
        Ok("234a"
            .parse()
            .map_err(|err: ParseIntError| err.extend_with(|_, e| e.set("code", 400)))?)
    }
}
```
