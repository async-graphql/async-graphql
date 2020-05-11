# 查询上下文(Context)

查询上下文(GqlContext)的主要作用是获取附加到Schema的全局数据，**需要注意的是，如果你的Resolve函数返回的数据借用了GqlContext内保存的数据，需要明确指定生命周期参数**。

下面是一个返回值借用GqlContext内数据的例子：

```rust
use async_graphql::*;

struct Query;

#[GqlObject]
impl Query {
    async fn borrow_from_context_data<'ctx'>(
        &self,
        ctx: &'ctx GqlContext<'_>
    ) -> &'ctx String {
        ctx.data::<String>
    }
}
```