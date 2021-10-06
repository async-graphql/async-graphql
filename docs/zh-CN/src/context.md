# 查询上下文(Context)

`Context`的主要目标是获取附加到`Schema`的全局数据或者与正在处理的实际查询相关的数据。

## 存储数据

在`Context`中你可以存放全局数据，例如环境变量、数据库连接池，以及你在每个查询中可能需要的任何内容。

数据必须实现`Send`和`Sync`。

你可以通过调用`ctx.data::<TypeOfYourData>()`来获取查询中的数据。

**主意：如果Resolver函数的返回值是从`Context`中借用的，则需要明确说明参数的生命周期。**

下面的例子展示了如何从`Context`中借用数据。

```rust
use async_graphql::*;

struct Query;

#[Object]
impl Query {
    async fn borrow_from_context_data<'ctx>(
        &self,
        ctx: &Context<'ctx>
    ) -> Result<&'ctx String> {
        ctx.data::<String>()
    }
}
```

### Schema数据

你可以在创建`Schema`时将数据放入上下文中，这对于不会更改的数据非常有用，例如连接池。

```rust
  let schema = Schema::build(Query::default(), Mutation::default(), EmptySubscription)
    .data(env_struct)
    .data(s3_storage)
    .data(db_core)
    .finish();
```

### 请求数据

你可以在执行请求时将数据放入上下文中，它对于身份验证数据很有用。

一个使用`warp`的小例子：

```rust
let graphql_post = warp::post()
  .and(warp::path("graphql"))
  .and(schema_filter)
  .and(a_warp_filter)
  ...
  .and_then( |schema: (Schema<Query, Mutation, Subscriptions>, async_graphql::Request), arg2: ArgType2 ...| async move {
    let (schema, request) = schema;
    let your_auth_data = auth_function_from_headers(headers).await?;
    let response = schema
      .execute(
        request
         .data(your_auth_data)
         .data(something_else)
      ).await;
      
    Ok(async_graphql_warp::Response::from(response))
  });
```

## HTTP头

使用`Context`你还可以插入或添加HTTP头。

```rust
#[Object]
impl Query {
    async fn greet(&self, ctx: &Context<'_>) -> String {
        // Headers can be inserted using the `http` constants
        let was_in_headers = ctx.insert_http_header(ACCESS_CONTROL_ALLOW_ORIGIN, "*");

        // They can also be inserted using &str
        let was_in_headers = ctx.insert_http_header("Custom-Header", "1234");

        // If multiple headers with the same key are `inserted` then the most recent
        // one overwrites the previous. If you want multiple headers for the same key, use
        // `append_http_header` for subsequent headers
        let was_in_headers = ctx.insert_http_header("Custom-Header", "Hello World");

        String::from("Hello world")
    }
}
```

## Selection / LookAhead

有时你想知道子查询中请求了哪些字段用于优化数据处理，则可以使用`ctx.fields()`读取查询中的字段，它将提供一个`SelectionField`，允许你在当前字段和子字段之间导航。

如果要跨查询或子查询执行搜索，则不必使用 `SelectionField` 手动执行此操作，可以使用 `ctx.look_ahead()` 来执行选择。

```rust
use async_graphql::*;

#[derive(SimpleObject)]
struct Detail {
    c: i32,
    d: i32,
}

#[derive(SimpleObject)]
struct MyObj {
    a: i32,
    b: i32,
    detail: Detail,
}

struct Query;

#[Object]
impl Query {
    async fn obj(&self, ctx: &Context<'_>) -> MyObj {
        if ctx.look_ahead().field("a").exists() {
            // This is a query like `obj { a }`
        } else if ctx.look_ahead().field("detail").field("c").exists() {
            // This is a query like `obj { detail { c } }`
        } else {
            // This query doesn't have `a`
        }
        unimplemented!()
    }
}
```
