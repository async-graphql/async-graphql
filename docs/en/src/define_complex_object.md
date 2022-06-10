# Object

Different from `SimpleObject`, `Object` must have a resolver defined for each field in its `impl`.

**A resolver function has to be asynchronous. The first argument has to be `&self`, the second is an optional `Context` and it is followed by field arguments.**

The resolver is used to get the value of the field. For example, you can query a database and return the result. **The return type of the function is the type of the field.** You can also return a `async_graphql::Result` to return an error if it occurs. The error message will then be sent as query result.

You may need access to global data in your query, for example a database connection pool.
When creating your `Schema`, you can use `SchemaBuilder::data` to configure the global data, and `Context::data` to configure `Context` data.
The following `value_from_db` function shows how to retrieve a database connection from `Context`.

```rust
# extern crate async_graphql;
# struct Data { pub name: String }
# struct DbConn {}
# impl DbConn {
#   fn query_something(&self, id: i64) -> std::result::Result<Data, String> { Ok(Data {name:"".into()})}
# }
# struct DbPool {}
# impl DbPool {
#   fn take(&self) -> DbConn { DbConn {} }    
# }
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
        ctx: &Context<'_>,
        #[graphql(desc = "Id of object")] id: i64
    ) -> Result<String> {
        let conn = ctx.data::<DbPool>()?.take();
        Ok(conn.query_something(id)?.name)
    }
}
```
