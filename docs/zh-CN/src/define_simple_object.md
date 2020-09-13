# 简单对象(SimpleObject)

简单对象是把Rust结构的所有字段都直接映射到GraphQL对象，不支持定义单独的Resolver函数。

下面的例子定义了一个名称为MyObject的对象，包含字段`a`和`b`，`c`由于标记为`#[field(skip)]`，所以不会映射到GraphQL。

**`a`和`b`字段都有描述信息，这会反映在GraphQL的内省信息中，它们采用了rustdoc和属性两种不同的写法，这两种写法`async-graphql`都支持。当两种写法同时存在时，属性的优先级更高。**

```rust
use async_graphql::*;

#[derive(GQLSimpleObject)]
struct MyObject {
    /// Value a
    a: i32,
    
    #[field(desc = "Value b")]
    b: i32,

    #[field(skip)]
    c: i32,
}
```
