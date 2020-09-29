# 错误处理

Resolver函数可以返回一个Result类型，以下是Result的定义：

```rust
type Result<T> = std::result::Result<T, Error>;
```

任何错误都能够被转换为`Error`，并且你还能扩展标准的错误信息。

下面是一个例子，解析一个输入的字符串到整数，当解析失败时返回错误，并且附加额外的错误信息。

```rust
use async_graphql::*;

struct Query;

#[Object]
impl Query {
    #[field]
    async fn parse_with_extensions(&self, input: String) -> Result<i32> {
        Ok("234a"
            .parse()
            .map_err(|err| err.extend_with(|_| json!({"code": 400})))?)
    }
}
```