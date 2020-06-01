# Object

Different from `SimpleObject`, `Object` must have Resolve defined for each field in `impl`.

**A resolver function has to be asynchronous. The first argument has to be `&self`, second being optional `Context` and followed by field arguments.**

Resolve is used to get the value of the field. You can query a database and return the result. **The return type of the function is the type of the field.** You can also return a `async_graphql::FieldResult` so to return an error if it occurs an error message will be sent to query result.

When querying a database, you may need a global data base connection pool.
When creating `Schema`,  you can use `SchemaBuilder::data` to setup `Schema` data, and `Context::data` to setup `Context`data.
The following `value_from_db` function showed how to retrieve a database connection from `Context`.

```rust
use async_graphql::*;

struct MyObject {
    value: i32,
}

#[Object]
impl MyObject {
    async fn value(&self) -> String {
        self.value.to_string()
    }

    async fn value_from_db(
        &self,
        ctx: &Context<'_'>,
        #[arg(desc = "Id of object")] id: i64
    ) -> FieldResult<String> {
        let conn = ctx.data::<DbPool>().take();
        Ok(conn.query_something(id)?.name)
    }
}
```
