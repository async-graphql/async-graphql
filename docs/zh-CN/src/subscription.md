# 订阅

订阅根对象和其它根对象定义稍有不同，它的Resolver函数总是返回一个 [Stream](https://docs.rs/futures-core/~0.3/futures_core/stream/trait.Stream.html) 或者`Result<Stream>`，而字段参数通常作为数据的筛选条件。

下面的例子订阅一个整数流，它每秒产生一个整数，参数`step`指定了整数的步长，默认为1。

```rust
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
