# 快速开始

## 添加依赖

```toml
[dependencies]
async-graphql = "1.9.0"
```

## 写一个Schema

一个GraphQL的Schema包含一个必须的查询(Query)根对象，可选的变更(Mutation)根对象和可选的订阅(Subscription)根对象，这些对象类型都是用Rust语言的struct来描述它们，结构的字段对应GraphQL对象的字段，但你需要用`#[field]`来修饰它，这样`Async-graphql`提供的过程宏才能够正确的识别它。

Async-graphql实现了常用数据类型到GraphQL类型的映射，例如`i32`, `f64`, `Option<T>`, `Vec<T>`等。同时，你也能够[扩展这些基础类型](custom_scalars.md)，基础数据类型在GraphQL里面称为标量。

下面是一个简单的例子，我们只提供一个查询，返回`a`和`b`的和。

```rust
use async_graphql::*;

struct Query;

#[Object]
impl Query {
    #[field(desc = "Returns the sum of a and b")]
    async fn add(a: i32, b: i32) -> i32 {
        a + b
    }
}

type MySchema = Schema<Query, EmptyMutation, EmptySubscription>;

```

## 执行查询