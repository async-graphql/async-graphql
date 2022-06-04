# Error extensions
To quote the [graphql-spec](https://spec.graphql.org/June2018/#example-fce18):
> GraphQL services may provide an additional entry to errors with key extensions.
> This entry, if set, must have a map as its value. This entry is reserved for implementer to add
> additional information to errors however they see fit, and there are no additional restrictions on
> its contents.

## Example 
I would recommend on checking out this [async-graphql example](https://github.com/async-graphql/examples/blob/master/actix-web/error-extensions/src/main.rs) as a quickstart.

## General Concept
In `async-graphql` all user-facing errors are cast to the `Error` type which by default provides
the error message exposed by `std::fmt::Display`. However, `Error` actually provides an additional information that can extend the error.

A resolver looks like this:

```rust
# extern crate async_graphql;
# use async_graphql::*;
# struct Query;
# #[Object]
# impl Query {
async fn parse_with_extensions(&self) -> Result<i32, Error> {
    Err(Error::new("MyMessage").extend_with(|_, e| e.set("details", "CAN_NOT_FETCH")))
}
# }
```

may then return a response like this:

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
Constructing new `Error`s by hand quickly becomes tedious. That is why `async-graphql` provides
two convenience traits for casting your errors to the appropriate `Error` with
extensions.

The easiest way to provide extensions to any error is by calling `extend_with` on the error.
This will on the fly convert any error into a `Error` with the given extension.

```rust
# extern crate async_graphql;
# use async_graphql::*;
# struct Query;
use std::num::ParseIntError;
# #[Object]
# impl Query {
async fn parse_with_extensions(&self) -> Result<i32> {
     Ok("234a"
         .parse()
         .map_err(|err: ParseIntError| err.extend_with(|_err, e| e.set("code", 404)))?)
}
# }
```

### Implementing ErrorExtensions for custom errors.
If you find yourself attaching extensions to your errors all over the place you might want to consider
implementing the trait on your custom error type directly.

```rust
# extern crate async_graphql;
# extern crate thiserror;
# use async_graphql::*;
#[derive(Debug, thiserror::Error)]
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
    fn extend(&self) -> Error {
        Error::new(format!("{}", self)).extend_with(|err, e| 
            match self {
              MyError::NotFound => e.set("code", "NOT_FOUND"),
              MyError::ServerError(reason) => e.set("reason", reason.clone()),
              MyError::ErrorWithoutExtensions => {}
          })
    }
}
```

This way you only need to call `extend` on your error to deliver the error message alongside the provided extensions.
Or further extend your error through `extend_with`.

```rust
# extern crate async_graphql;
# extern crate thiserror;
# use async_graphql::*;
# #[derive(Debug, thiserror::Error)]
# pub enum MyError {
#     #[error("Could not find resource")]
#     NotFound,
# 
#     #[error("ServerError")]
#     ServerError(String),
# 
#     #[error("No Extensions")]
#     ErrorWithoutExtensions,
# }
# struct Query;
# #[Object]
# impl Query {
async fn parse_with_extensions_result(&self) -> Result<i32> {
    // Err(MyError::NotFound.extend())
    // OR
    Err(MyError::NotFound.extend_with(|_, e| e.set("on_the_fly", "some_more_info")))
}
# }
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
This trait enables you to call `extend_err` directly on results. So the above code becomes less verbose.

```rust,ignore
# // @todo figure out why this example does not compile!
# extern crate async_graphql;
use async_graphql::*;
# struct Query;
# #[Object]
# impl Query {
async fn parse_with_extensions(&self) -> Result<i32> {
     Ok("234a"
         .parse()
         .extend_err(|_, e| e.set("code", 404))?)
}
# }
```
### Chained extensions
Since `ErrorExtensions` and `ResultExt` are implemented for any type `&E where E: std::fmt::Display`
we can chain the extension together.


```rust
# extern crate async_graphql;
use async_graphql::*;
# struct Query;
# #[Object]
# impl Query {
async fn parse_with_extensions(&self) -> Result<i32> {
    match "234a".parse() {
        Ok(n) => Ok(n),
        Err(e) => Err(e
            .extend_with(|_, e| e.set("code", 404))
            .extend_with(|_, e| e.set("details", "some more info.."))
            // keys may also overwrite previous keys...
            .extend_with(|_, e| e.set("code", 500))),
    }
}
# }
```
Expected response:

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

### Pitfalls
Rust does not provide stable trait specialization yet.
That is why `ErrorExtensions` is actually implemented for `&E where E: std::fmt::Display`
instead of `E: std::fmt::Display`. Some specialization is provided through
[Autoref-based stable specialization](https://github.com/dtolnay/case-studies/blob/master/autoref-specialization/README.md).
The disadvantage is that the below code does **NOT** compile:

```rust,ignore,does_not_compile
async fn parse_with_extensions_result(&self) -> Result<i32> {
    // the trait `error::ErrorExtensions` is not implemented
    // for `std::num::ParseIntError`
    "234a".parse().extend_err(|_, e| e.set("code", 404))
}
```

however this does:

```rust,ignore,does_not_compile
async fn parse_with_extensions_result(&self) -> Result<i32> {
    // does work because ErrorExtensions is implemented for &ParseIntError
    "234a"
      .parse()
      .map_err(|ref e: ParseIntError| e.extend_with(|_, e| e.set("code", 404)))
}
```
