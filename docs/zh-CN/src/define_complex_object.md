# 对象 (Object)

和简单对象不同，对象必须为所有的字段定义 Resolver 函数，Resolver 函数定义在 impl 块中。

**一个 Resolver 函数必须是异步的，它的第一个参数必须是`&self`，第二个参数是可选的`Context`，接下来是字段的参数。**

Resolver 函数用于计算字段的值，你可以执行一个数据库查询，并返回查询结果。**函数的返回值是字段的类型**，你也可以返回一个`async_graphql::Result`类型，这样能够返回一个错误，这个错误信息将输出到查询结果中。

在查询数据库时，你可能需要一个数据库连接池对象，这个对象是个全局的，你可以在创建 Schema 的时候，用`SchemaBuilder::data`函数设置`Schema`数据，用`Context::data`函数设置`Context`数据。下面的`value_from_db`字段展示了如何从`Context`中获取一个数据库连接。

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

