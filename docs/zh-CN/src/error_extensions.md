＃ 错误扩展

引用 [graphql-spec](https://spec.graphql.org/June2018/#example-fce18)

> GraphQL服务可以通过扩展提供错误的附加条目。
> 该条目（如果设置）必须是一个映射作为其值，用于附加错误的其它信息。

## 示例

我建议您查看此 [错误扩展示例](https://github.com/async-graphql/examples/blob/master/actix-web/error-extensions/src/main.rs) 作为快速入门。

## 一般概念

在`Async-graphql`中，所有面向用户的错误都强制转换为`FieldError`类型，默认情况下会提供
由`std:::fmt::Display`暴露的错误消息。但是，`FieldError`实际上提供了一个额外的
字段`Option<serde_json::Value>`，如果它是一个`serde_json::Map`，则可以扩展错误的信息。

Resolver函数类似这样：

```rust
async fn parse_with_extensions(&self) -> Result<i32, FieldError> {
    let my_extension = json!({ "details": "CAN_NOT_FETCH" });
    Err(FieldError("MyMessage", Some(my_extension)))
 }
```

然后可以返回如下响应：

```json
{
  "errors": [
    {
      "message": "MyMessage",
      "locations": [ ... ],
      "path": [ ... ],
      "extensions": {
        "details": "CAN_NOT_FETCH",
      }
    }
  ]
}
```


## ErrorExtensions

手动构造新的`FieldError`很麻烦。这就是为什么`Async-graphql`提供
两个方便特性，可将您的错误转换为适当的`FieldError`扩展。

扩展任何错误的最简单方法是对错误调用`extend_with`。
这将把任何错误转换为具有给定扩展信息的`FieldError`。

```rust
use std::num::ParseIntError;
async fn parse_with_extensions(&self) -> FieldResult<i32> {
     Ok("234a"
         .parse()
         .map_err(|err: ParseIntError| err.extend_with(|_err| json!({"code": 404})))?)
 }
```

### 为自定义错误实现ErrorExtensions

你也可以给自己的错误类型实现`ErrorExtensions`:


```rust
#[macro_use]
extern crate thiserror;

#[derive(Debug, Error)]
pub enum MyError {
    #[error("Could not find resource")]
    NotFound,

    #[error("ServerError")]
    ServerError(String),

    #[error("No Extensions")]
    ErrorWithoutExtensions,
}

impl ErrorExtensions for MyError {
    // lets define our base extensions
    fn extend(&self) -> FieldError {
        let extensions = match self {
            MyError::NotFound => json!({"code": "NOT_FOUND"}),
            MyError::ServerError(reason) => json!({ "reason": reason }),
            MyError::ErrorWithoutExtensions => {
                json!("This will be ignored since it does not represent an object.")
            }
        };

        FieldError(format!("{}", self), Some(extensions))
    }
}
```

您只需要对错误调用`extend`即可将错误与其提供的扩展信息一起传递，或者通过`extend_with`进一步扩展错误信息。

```rust
async fn parse_with_extensions_result(&self) -> FieldResult<i32> {
    // Err(MyError::NotFound.extend())
    // OR
    Err(MyError::NotFound.extend_with(|_| json!({ "on_the_fly": "some_more_info" })))
}
```

```json
{
  "errors": [
    {
      "message": "NotFound",
      "locations": [ ... ],
      "path": [ ... ],
      "extensions": {
        "code": "NOT_FOUND",
        "on_the_fly": "some_more_info"
      }
    }
  ]
}
```

## ResultExt
这个特质使您可以直接在结果上调用`extend_err`。因此上面的代码不再那么冗长。

```rust
use async_graphql::*;
async fn parse_with_extensions(&self) -> FieldResult<i32> {
     Ok("234a"
         .parse()
         .extend_err(|_| json!({"code": 404}))?)
 }

```

### 链式调用

由于对所有`&E where E: std::fmt::Display`实现了`ErrorExtensions`和`ResultsExt`，我们可以将扩展链接在一起。

```rust
use async_graphql::*;
async fn parse_with_extensions(&self) -> FieldResult<i32> {
    match "234a".parse() {
        Ok(n) => Ok(n),
        Err(e) => Err(e
            .extend_with(|_| json!({"code": 404}))
            .extend_with(|_| json!({"details": "some more info.."}))
            // keys may also overwrite previous keys...
            .extend_with(|_| json!({"code": 500}))),
    }
}
```

响应：

```json
{
  "errors": [
    {
      "message": "MyMessage",
      "locations": [ ... ],
      "path": [ ... ],
      "extensions": {
      	"details": "some more info...",
        "code": 500,
      }
    }
  ]
}
```

### 缺陷

Rust的稳定版本还未提供特化功能，这就是为什么`ErrorExtensions`为`&E where E: std::fmt::Display`实现，代替`E：std::fmt::Display`通过提供一些特化功能。

[Autoref-based stable specialization](https://github.com/dtolnay/case-studies/blob/master/autoref-specialization/README.md).

缺点是下面的代码**不能**编译：

```rust,ignore,does_not_compile
async fn parse_with_extensions_result(&self) -> FieldResult<i32> {
    // the trait `error::ErrorExtensions` is not implemented
    // for `std::num::ParseIntError`
    "234a".parse().extend_err(|_| json!({"code": 404}))
}
```

但这可以通过编译：

```rust,ignore,does_not_compile
async fn parse_with_extensions_result(&self) -> FieldResult<i32> {
    // does work because ErrorExtensions is implemented for &ParseIntError
    "234a"
      .parse()
      .map_err(|ref e: ParseIntError| e.extend_with(|_| json!({"code": 404})))
}
```

