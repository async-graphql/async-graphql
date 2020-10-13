# 简单对象(SimpleObject)

简单对象是把Rust结构的所有字段都直接映射到GraphQL对象，不支持定义单独的Resolver函数。

下面的例子定义了一个名称为MyObject的对象，包含字段`a`和`b`，`c`由于标记为`#[graphql(skip)]`，所以不会映射到GraphQL。

```rust
use async_graphql::*;

#[derive(SimpleObject)]
struct MyObject {
    /// Value a
    a: i32,
    
    /// Value b
    b: i32,

    #[graphql(skip)]
    c: i32,
}
```
