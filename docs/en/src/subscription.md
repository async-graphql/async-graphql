# Subscription

The definition of the subscription root object is slightly different from other root objects. Its resolver function always returns a [Stream](https://docs.rs/futures-core/~0.3/futures_core/stream/trait.Stream.html) or `Result<Stream>`, and the field parameters are usually used as data filtering conditions.

The following example subscribes to an integer stream, which generates one integer per second. The parameter `step` specifies the integer step size with a default of 1.

```rust
# extern crate async_graphql;
# use std::time::Duration;
# use async_graphql::futures_util::stream::Stream;
# use async_graphql::futures_util::StreamExt;
# extern crate tokio_stream;
# extern crate tokio;
use async_graphql::*;

struct Subscription;

#[Subscription]
impl Subscription {
    async fn integers(&self, #[graphql(default = 1)] step: i32) -> impl Stream<Item = i32> {
        let mut value = 0;
        tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(Duration::from_secs(1)))
            .map(move |_| {
                value += step;
                value
            })
    }
}
```
