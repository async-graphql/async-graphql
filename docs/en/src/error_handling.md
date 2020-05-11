# Error handling

Resolve can return a `GqlFieldResult`, following is the definition:

```rust
type GqlFieldResult<T> = std::result::Result<T, FieldError>;
```

Any `Error` that implements `std::fmt::Display` can be converted to `FieldError` and you can extend error message.

Following example shows how to parse an input string to integer. When parsing failed, it would return error and attach error message.
See [ErrorExtensions](error_extensions.md) sections of this book for more details.

```rust
use async_graphql::*;

struct Query;

#[GqlObject]
impl Query {
    #[field]
    async fn parse_with_extensions(&self, input: String) -> GqlFieldResult<i32> {
        Ok("234a"
            .parse()
            .map_err(|err| err.extend_with(|_| json!({"code": 400})))?)
    }
}
```
