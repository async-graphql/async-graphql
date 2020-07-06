# 查询上下文(Context)

查询上下文(Context)的主要作用是获取附加到Schema的全局数据，**需要注意的是，如果你的Resolver函数返回的数据借用了Context内保存的数据，需要明确指定生命周期参数**。

下面是一个返回值借用Context内数据的例子：

```rust
use async_graphql::*;

struct Query;

#[Object]
impl Query {
    async fn borrow_from_context_data<'ctx>(
        &self,
        ctx: &'ctx Context<'_>
    ) -> FieldResult<&'ctx String> {
        ctx.data::<String>()
    }
}
```
