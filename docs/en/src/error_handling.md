# Error handling

Resolve can return a `FieldResult`, following is the definition:

```rust
type FieldResult<T> = std::result::Result<T, FieldError>;
```

<!--TODO: 扩展标准的错误输出? -->
Any `Error` can be converted to `FieldError` and you can extend error message.

Following example shows how to parse an input string to integer. When parsing failed, it would return error and attach error message.

```rust
use async_graphql::*;

struct Query;

#[Object]
impl Query {
    #[field]
    async fn parse_with_extensions(&self, input: String) -> FieldResult<i32> {
        Ok("234a"
            .parse()
            .map_err(|err| err.extend_with(|_| json!({"code": 400})))?)
    }
}
```