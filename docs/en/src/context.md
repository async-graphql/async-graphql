# Context

The main goal of `GqlContext` is to acquire global data attached to Schema. **Note that if the return value of resolve function is borrowed from `GqlContext`, you need to explictly state the lifetime of the argument.**

The following example shows how to borrow data in `GqlContext`.

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