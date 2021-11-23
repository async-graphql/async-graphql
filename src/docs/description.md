Attach a description to `Object`, `Scalar` or `Subscription`.

The three types above use the rustdoc on the implementation block as
the GraphQL type description, but if you want to use the rustdoc on the
type declaration as the GraphQL type description, you can use that derived macro.

# Examples

```rust
use async_graphql::*;

/// This is MyObj
#[derive(Description, Default)]
struct MyObj;

#[Object(use_type_description)]
impl MyObj {
    async fn value(&self) -> i32 {
        100
    }
}

#[derive(SimpleObject, Default)]
struct Query {
    obj: MyObj,
}

# tokio::runtime::Runtime::new().unwrap().block_on(async move {
let schema = Schema::new(Query::default(), EmptyMutation, EmptySubscription);
assert_eq!(
    schema
        .execute(r#"{ __type(name: "MyObj") { description } }"#)
        .await
        .data,
    value!({
        "__type": { "description": "This is MyObj" }
    })
);
# });
```
