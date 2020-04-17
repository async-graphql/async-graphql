# 简单对象(SimpleObject)

简单对象是把Rust结构的所有字段都直接映射到GraphQL对象，不支持定义单独的Resolve函数。

下面的例子定义了一个名称为MyObject的对象，包含字段`a`和`b`，`c`由于没有`#[field]`标记，所以不会映射到GraphQL。

```rust
use async_graphql::*;

#[SimpleObject]
struct MyObject {
    #[field]
    a: i32,

    #[field]
    b: i32,

    c: i32,
}
```
