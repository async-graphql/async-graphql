# 查询缓存控制

生产环境下通常依赖缓存来提高性能。

一个 GraphQL 查询会调用多个 Resolver 函数，每个 Resolver 函数都能够具有不同的缓存定义。有的可能缓存几秒钟，有的可能缓存几个小时，有的可能所有用户都相同，有的可能每个会话都不同。

`Async-graphql`提供一种机制允许定义结果的缓存时间和作用域。

你可以在**对象**上定义缓存参数，也可以在**字段**上定义，下面的例子展示了缓存控制参数的两种用法。

你可以用`max_age`参数来控制缓存时长（单位是秒），也可以用`public`和`private`来控制缓存的作用域，当你不指定时，作用域默认是`public`。

`Async-graphql`查询时会合并所有缓存控制指令的结果，`max_age`取最小值。如果任何对象或者字段的作用域为`private`，则其结果的作用域为`private`，否则为`public`。

我们可以从查询结果`QueryResponse`中获取缓存控制合并结果，并且调用`CacheControl::value`来获取对应的 HTTP 头。

```rust
# extern crate async_graphql;
# use async_graphql::*;
# struct Query;
#[Object(cache_control(max_age = 60))]
impl Query {
    #[graphql(cache_control(max_age = 30))]
    async fn value1(&self) -> i32 {
        1
    }

    #[graphql(cache_control(private))]
    async fn value2(&self) -> i32 {
        2
    }

    async fn value3(&self) -> i32 {
        3
    }
}
```

下面是不同的查询对应不同缓存控制结果：

```graphql
# max_age=30
{ value1 }
```

```graphql
# max_age=30, private
{ value1 value2 }
```

```graphql
# max_age=60
{ value3 }
```
