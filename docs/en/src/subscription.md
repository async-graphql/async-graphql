# Subscription

The definition of the subscription root object is slightly different from other root objects. Its resolver function always returns a [Stream](https://docs.rs/futures-core/~0.3/futures_core/stream/trait.Stream.html), and the field parameters are usually used as data filtering conditions.

The following example subscribes to an integer stream, which generates one integer per second. The parameter `step` specifies the integer step size with a default of 1.

```rust
use async_graphql::*;

struct Subscription;

#[Subscription]
impl Subscription {
    async fn integers(&self, #[arg(default = "1")] step: i32) -> impl Stream<Item = i32> {
        let mut value = 0;
        tokio::time::interval(Duration::from_secs(1)).map(move |_| {
            value += step;
            value
        })
    }
}
```
