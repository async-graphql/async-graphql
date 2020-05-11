# Object

Different from `SimpleObject`, `Object` must have Resolve defined for each field in `impl`.

**A Resolve function has to be asynchronous. The first argument has to be `&self`, second being optional `GqlContext` and followed by field arguments.**

Resolve is used to get the value of the field. You can query a database and return the result. **The return type of the function is the type of the field.** You can also return a `async_graphql::GqlFieldResult` so to return an error if it occrs and error message will be send to query result.

When querying a database, you may need a global data base connection pool.
When creating `GqlSchema`,  you can use `SchemaBuilder::data` to setup `GqlSchema` data, and `GqlContext::data` to setup `GqlContext`data.
The following `value_from_db` function showed how to retrive a database connection from `GqlContext`.

```rust
use async_graphql::*;

struct MyObject {
    value: i32,
}

#[GqlObject]
impl MyObject {
    async fn value(&self) -> String {
        self.value.to_string()
    }

    async fn value_from_db(
        &self,
        ctx: &GqlContext<'_'>,
        #[arg(desc = "Id of object")] id: i64
    ) -> GqlFieldResult<String> {
        let conn = ctx.data::<DbPool>().take();
        Ok(conn.query_something(id)?.name)
    }
}
```