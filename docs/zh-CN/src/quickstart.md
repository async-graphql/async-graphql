# 快速开始

## 添加依赖

```toml
[dependencies]
async-graphql = "4.0"
async-graphql-actix-web = "4.0" # 如果你需要集成到Actix-web
async-graphql-warp = "4.0" # 如果你需要集成到Warp
async-graphql-tide = "4.0" # 如果你需要集成到Tide
```

## 写一个Schema

一个GraphQL的Schema包含一个必须的查询(Query)根对象，可选的变更(Mutation)根对象和可选的订阅(Subscription)根对象，这些对象类型都是用Rust语言的结构来描述它们，结构的字段对应GraphQL对象的字段。

Async-graphql实现了常用数据类型到GraphQL类型的映射，例如`i32`, `f64`, `Option<T>`, `Vec<T>`等。同时，你也能够[扩展这些基础类型](custom_scalars.md)，基础数据类型在GraphQL里面称为标量。

下面是一个简单的例子，我们只提供一个查询，返回`a`和`b`的和。

```rust
# extern crate async_graphql;
use async_graphql::*;

struct Query;

#[Object]
impl Query {
    /// Returns the sum of a and b
    async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }
}

```

## 执行查询

在我们这个例子里面，只有Query，没有Mutation和Subscription，所以我们用`EmptyMutation`和`EmptySubscription`来创建Schema，然后调用`Schema::execute`来执行查询。

```rust
# extern crate async_graphql;
# use async_graphql::*;
#
# struct Query;
# #[Object]
# impl Query {
#   async fn version(&self) -> &str { "1.0" }    
# }
# async fn other() {
let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
let res = schema.execute("{ add(a: 10, b: 20) }").await;
# }
```

## 把查询结果输出为JSON

```rust,ignore
let json = serde_json::to_string(&res);
```

## 和Web Server的集成

请参考https://github.com/async-graphql/examples。
