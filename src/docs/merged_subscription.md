Define a merged subscription with multiple subscription types.

*[See also the Book](https://async-graphql.github.io/async-graphql/en/merging_objects.html).*

# Macro attributes

| Attribute | description                                                                                                                                                 | Type   | Optional |
|-----------|-------------------------------------------------------------------------------------------------------------------------------------------------------------|--------|----------|
| name      | Object name                                                                                                                                                 | string | Y        |
| name_type | If `true`, the object name will be specified from [`async_graphql::TypeName`](https://docs.rs/async-graphql/latest/async_graphql/trait.TypeName.html) trait | bool   | Y        |
| extends   | Add fields to an entity that's defined in another service                                                                                                   | bool   | Y        |
| visible   | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).*             | bool   | Y        |
| visible   | Call the specified function. If the return value is `false`, it will not be displayed in introspection.                                                     | string | Y        |

# Examples

```rust
use async_graphql::*;
use futures_util::stream::Stream;

#[derive(Default)]
struct Subscription1;

#[Subscription]
impl Subscription1 {
    async fn events1(&self) -> impl Stream<Item = i32> {
        futures_util::stream::iter(0..10)
    }
}

#[derive(Default)]
struct Subscription2;

#[Subscription]
impl Subscription2 {
    async fn events2(&self) -> impl Stream<Item = i32> {
        futures_util::stream::iter(10..20)
   }
}

#[derive(MergedSubscription, Default)]
struct Subscription(Subscription1, Subscription2);
```
