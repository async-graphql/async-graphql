Define a NewType Scalar

It also implements `From<InnerType>` and `Into<InnerType>`.

# Macro attributes

| Attribute                                    | description                                                                                                                                                            | Type   | Optional |
|----------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------|----------|
| name                                         | If this attribute is provided then define a new scalar, otherwise it is just a transparent proxy for the internal scalar.                                              | string | Y        |
| visible(Only valid for new scalars)          | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).*                        | bool   | Y        |
| visible(Only valid for new scalars)          | Call the specified function. If the return value is `false`, it will not be displayed in introspection.                                                                | string | Y        |
| specified_by_url(Only valid for new scalars) | Provide a specification URL for this scalar type, it must link to a human-readable specification of the data format, serialization and coercion rules for this scalar. | string | Y        |
| inaccessible                                 | Indicate that an object is not accessible from a supergraph when using Apollo Federation                                                                               | bool   | Y        |
| tag                                          | Arbitrary string metadata that will be propagated to the supergraph when using Apollo Federation. This attribute is repeatable                                         | string | Y        |

# Examples

## Use the original scalar name

```rust
use async_graphql::*;

#[derive(NewType)]
struct Weight(f64);

struct Query;

#[Object]
impl Query {
    async fn value(&self) -> Weight {
        Weight(1.234)
    }
}

// Test conversion
let weight: Weight = 10f64.into();
let weight_f64: f64 = weight.into();

# tokio::runtime::Runtime::new().unwrap().block_on(async move {
let schema = Schema::build(Query, EmptyMutation, EmptySubscription).data("hello".to_string()).finish();

let res = schema.execute("{ value }").await.into_result().unwrap().data;
assert_eq!(res, value!({
    "value": 1.234,
}));

let res = schema.execute(r#"
{
    __type(name: "Query") {
        fields {
            name type {
                kind
                ofType { name }
            }
        }
    }
}"#).await.into_result().unwrap().data;
assert_eq!(res, value!({
    "__type": {
        "fields": [{
            "name": "value",
            "type": {
                "kind": "NON_NULL",
                "ofType": {
                    "name": "Float"
                }
            }
        }]
    }
}));
# });
```

## Define a new scalar

```rust
use async_graphql::*;

/// Widget NewType
#[derive(NewType)]
#[graphql(name)] // or: #[graphql(name = true)], #[graphql(name = "Weight")]
struct Weight(f64);

struct Query;

#[Object]
impl Query {
    async fn value(&self) -> Weight {
        Weight(1.234)
    }
}

# tokio::runtime::Runtime::new().unwrap().block_on(async move {
let schema = Schema::build(Query, EmptyMutation, EmptySubscription).data("hello".to_string()).finish();

let res = schema.execute("{ value }").await.into_result().unwrap().data;
assert_eq!(res, value!({
    "value": 1.234,
}));

let res = schema.execute(r#"
{
    __type(name: "Query") {
        fields {
            name type {
                kind
                ofType { name }
            }
        }
    }
}"#).await.into_result().unwrap().data;
assert_eq!(res, value!({
    "__type": {
        "fields": [{
            "name": "value",
            "type": {
                "kind": "NON_NULL",
                "ofType": {
                    "name": "Weight"
                }
            }
        }]
    }
}));

assert_eq!(schema.execute(r#"{ __type(name: "Weight") { name description } }"#).
    await.into_result().unwrap().data, value!({
        "__type": {
            "name": "Weight", "description": "Widget NewType"
        }
    }));
# });
```
