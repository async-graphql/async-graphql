# How extensions are defined

An `async-graphql` extension is defined by implementing the trait `Extension` associated. The `Extension` trait allows you to insert custom code to some several steps used to respond to GraphQL's queries through `async-graphql`. With `Extensions`, your application can hook into the GraphQL's requests lifecycle to add behaviors about incoming requests or outgoing response.

`Extensions` are a lot like middleware from other frameworks, be careful when using those: when you use an extension **it'll be run for every GraphQL request**.

Across every step, you'll have the `ExtensionContext` supplied with data about your current request execution. Feel free to check how it's constructed in the code, documentation about it will soon come.

## A word about middleware

For those who don't know, let's dig deeper into what is a middleware:

```rust,ignore
async fn middleware(&self, ctx: &ExtensionContext<'_>, next: NextMiddleware<'_>) -> MiddlewareResult {
  // Logic to your middleware.

  /*
   * Final step to your middleware, we call the next function which will trigger
   * the execution of the next middleware. It's like a `callback` in JavaScript.
   */
  next.run(ctx).await
}
```

As you have seen, a `Middleware` is only a function calling the next function at the end, but we could also do a middleware with the `next.run` function at the start. This is where it's becoming tricky: depending on where you put your logic and where is the `next.run` call, your logic won't have the same execution order.


Depending on your logic code, you'll want to process it before or after the `next.run` call. If you need more information about middlewares, there are a lot of things on the web.

## Processing of a query

There are several steps to go to process a query to completion, you'll be able to create extension based on these hooks.

### request

First, when we receive a request, if it's not a subscription, the first function to be called will be `request`, it's the first step, it's the function called at the incoming request, and it's also the function which will output the response to the user.

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

Depending on where you put your logic code, it'll be executed at the beginning or at the end of the query being processed.


```rust
# extern crate async_graphql;
# use async_graphql::*;
# use async_graphql::extensions::*;
# struct MyMiddleware;
# #[async_trait::async_trait]
# impl Extension for MyMiddleware {
async fn request(&self, ctx: &ExtensionContext<'_>, next: NextRequest<'_>) -> Response {
    // The code here will be run before the prepare_request is executed.
    let result = next.run(ctx).await;
    // The code after the completion of this future will be after the processing, just before sending the result to the user.
    result
}
# }
```

### prepare_request

Just after the `request`, we will have the `prepare_request` lifecycle, which will be hooked. 

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
    // The code here will be un before the prepare_request is executed, just after the request lifecycle hook.
    let result = next.run(ctx, request).await;
    // The code here will be run just after the prepare_request
    result
}
# }
```

### parse_query

The `parse_query` will create a GraphQL `ExecutableDocument` on your query, it'll check if the query is valid for the GraphQL Spec. Usually the implemented spec in `async-graphql` tends to be the last stable one (October2021).

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

### validation

The `validation` step will check (depending on your `validation_mode`) rules the query should abide to and give the client data about why the query is not valid.

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

### execute

The `execution` step is a huge one, it'll start the execution of the query by calling each resolver concurrently for a `Query` and serially for a `Mutation`.

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
    // Before starting resolving the whole query
    let result = next.run(ctx, operation_name).await;
    // After resolving the whole query
    result
}
# }
````

### resolve

The `resolve` step is launched for each field.

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
    // Logic before resolving the field
    let result = next.run(ctx, info).await;
    // Logic after resolving the field
    result
}
# }
```

### subscribe

The `subscribe` lifecycle has the same behavior as the `request` but for a `Subscritpion`.

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
