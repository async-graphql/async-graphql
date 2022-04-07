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

#### Errors in subscriptions

Errors can be returned from subscription resolvers as well, using a return type of the form:
```rust
async fn my_subscription_resolver(&self) -> impl Stream<Item = Result<MyItem, MyError>> { ... }
```

Note however that the `MyError` struct must have `Clone` implemented, due to the restrictions placed by the `Subscription` macro. One way to accomplish this is by creating a custom error type, with `#[derive(Clone)]`, as [seen here](https://github.com/async-graphql/async-graphql/issues/845#issuecomment-1090933464).
