# 如何定义扩展

`async-graphql` 扩展是通过实现 `Extension` trait 来定义的。 `Extension` trait 允许你将自定义代码插入到执行 GraphQL 查询的步骤中。

`Extensions` 很像来自其他框架的中间件，使用它们时要小心：当你使用扩展时**它对每个 GraphQL 请求生效**。

## 一句话解释什么是中间件

让我们了解什么是中间件：

```rust,ignore
async fn middleware(&self, ctx: &ExtensionContext<'_>, next: NextMiddleware<'_>) -> MiddlewareResult {
  // 你的中间件代码

  /*
   * 调用 next.run 函数执行下个中间件的逻辑
   */
  next.run(ctx).await
}
```

如你所见，`middleware` 只是在末尾调用 next 函数的函数。但我们也可以在开头使用 `next.run` 来实现中间件。这就是它变得棘手的地方：根据你放置逻辑的位置以及`next.run`调用的位置，你的逻辑将不会具有相同的执行顺序。

根据你代码，你需要在 `next.run` 调用之前或之后处理它。如果你需要更多关于中间件的信息，网上有很多。

## 查询的处理

查询的每个阶段都有回调，你将能够基于这些回调创建扩展。

### 请求

首先，当我们收到一个请求时，如果它不是订阅，第一个被调用的函数将是 `request`，它在传入请求时调用，并输出结果给客户端。

Default implementation for `request`:

```rust
# extern crate async_graphql;
# use async_graphql::*;
# use async_graphql::extensions::*;
# struct MyMiddleware;
# #[async_trait::async_trait]
# impl Extension for MyMiddleware {
async fn request(&self, ctx: &ExtensionContext<'_>, next: NextRequest<'_>) -> Response {
    next.run(ctx).await
}
# }
```

根据你放置逻辑代码的位置，它将在正在查询执行的开头或结尾执行。


```rust
# extern crate async_graphql;
# use async_graphql::*;
# use async_graphql::extensions::*;
# struct MyMiddleware;
# #[async_trait::async_trait]
# impl Extension for MyMiddleware {
async fn request(&self, ctx: &ExtensionContext<'_>, next: NextRequest<'_>) -> Response {
    // 此处的代码将在执行 prepare_request 之前运行。
    let result = next.run(ctx).await;
    // 此处的代码将在把结果发送给客户端之前执行
    result
}
# }
```

### 准备查询

在 `request` 之后，将调用`prepare_request`，你可以在此处对请求做一些转换。

```rust
# extern crate async_graphql;
# use async_graphql::*;
# use async_graphql::*;
# use async_graphql::extensions::*;
# struct MyMiddleware;
# #[async_trait::async_trait]
# impl Extension for MyMiddleware {
async fn prepare_request(
    &self,
    ctx: &ExtensionContext<'_>,
    request: Request,
    next: NextPrepareRequest<'_>,
) -> ServerResult<Request> {
    // 此处的代码在 prepare_request 之前执行
    let result = next.run(ctx, request).await;
    // 此处的代码在 prepare_request 之后执行
    result
}
# }
```

### 解析查询

`parse_query` 将解析查询语句并生成 GraphQL `ExecutableDocument`，并且检查查询是否遵循 GraphQL 规范。通常，`async-graphql` 遵循最后一个稳定的规范（October2021）。

```rust
# extern crate async_graphql;
# use async_graphql::*;
# use async_graphql::extensions::*;
# use async_graphql::parser::types::ExecutableDocument;
# struct MyMiddleware;
# #[async_trait::async_trait]
# impl Extension for MyMiddleware {
/// Called at parse query.
async fn parse_query(
    &self,
    ctx: &ExtensionContext<'_>,
    // The raw query
    query: &str,
    // The variables
    variables: &Variables,
    next: NextParseQuery<'_>,
) -> ServerResult<ExecutableDocument> {
    next.run(ctx, query, variables).await
}
# }
```

### 校验

`validation` 步骤将执行查询校验（取决于你指定的 `validation_mode`），并向客户端提供有关查询无效的原因。

```rust
# extern crate async_graphql;
# use async_graphql::*;
# use async_graphql::extensions::*;
# struct MyMiddleware;
# #[async_trait::async_trait]
# impl Extension for MyMiddleware {
/// Called at validation query.
async fn validation(
  &self,
  ctx: &ExtensionContext<'_>,
  next: NextValidation<'_>,
) -> Result<ValidationResult, Vec<ServerError>> {
  next.run(ctx).await
}
# }
```

### 执行

`execution` 步骤是一个很大的步骤，它将并发执行`Query`，或者顺序执行`Mutation`。

```rust
# extern crate async_graphql;
# use async_graphql::*;
# use async_graphql::extensions::*;
# struct MyMiddleware;
# #[async_trait::async_trait]
# impl Extension for MyMiddleware {
/// Called at execute query.
async fn execute(
    &self,
    ctx: &ExtensionContext<'_>,
    operation_name: Option<&str>,
    next: NextExecute<'_>,
) -> Response {
    // 此处的代码在执行完整查询之前执行
    let result = next.run(ctx, operation_name).await;
    // 此处的代码在执行完整查询之后执行
    result
}
# }
````

### resolve

为每个字段执行`resolve`.

```rust
# extern crate async_graphql;
# use async_graphql::*;
# use async_graphql::extensions::*;
# struct MyMiddleware;
# #[async_trait::async_trait]
# impl Extension for MyMiddleware { 
/// Called at resolve field.
async fn resolve(
    &self,
    ctx: &ExtensionContext<'_>,
    info: ResolveInfo<'_>,
    next: NextResolve<'_>,
) -> ServerResult<Option<Value>> {
    // resolve 字段之前
    let result = next.run(ctx, info).await;
    // resolve 字段之后
    result
}
# }
```

### 订阅

`subscribe`的行为和`request`很像，只是专门用于订阅查询。

```rust
# extern crate async_graphql;
# use async_graphql::*;
# use async_graphql::extensions::*;
# use futures_util::stream::BoxStream;
# struct MyMiddleware;
# #[async_trait::async_trait]
# impl Extension for MyMiddleware {
/// Called at subscribe request.
fn subscribe<'s>(
    &self,
    ctx: &ExtensionContext<'_>,
    stream: BoxStream<'s, Response>,
    next: NextSubscribe<'_>,
) -> BoxStream<'s, Response> {
    next.run(ctx, stream)
}
# }
``` 
