# Error handling

Resolve can return a `Result`, which has the following definition:

```rust
type Result<T> = std::result::Result<T, Error>;
```

Any `Error` that implements `std::fmt::Display` can be converted to `Error` and you can extend the error message.

The following example shows how to parse an input string to an integer. When parsing fails, it will return an error and attach an error message.
See the [Error Extensions](error_extensions.md) section of this book for more details.

```rust
use async_graphql::*;

struct Query;

#[Object]
impl Query {
    async fn parse_with_extensions(&self, input: String) -> Result<i32> {
        Ok("234a"
            .parse()
            .map_err(|err: ParseIntError| err.extend_with(|_| json!({"code": 400})))?)
    }
}
```
