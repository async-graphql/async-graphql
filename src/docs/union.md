Define a GraphQL union

*[See also the Book](https://async-graphql.github.io/async-graphql/en/define_union.html).*

# Macro attributes

| Attribute | description                                                                                                                                     | Type   | Optional |
|-----------|-------------------------------------------------------------------------------------------------------------------------------------------------|--------|----------|
| name      | Object name                                                                                                                                     | string | Y        |
| visible   | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool   | Y        |
| visible   | Call the specified function. If the return value is `false`, it will not be displayed in introspection.                                         | string | Y        |

# Item attributes

| Attribute    | description                              | Type     | Optional |
|--------------|------------------------------------------|----------|----------|
| flatten      | Similar to serde (flatten)               | boolean  | Y        |

# Define a union

Define TypeA, TypeB, ... as MyUnion

```rust
use async_graphql::*;

#[derive(SimpleObject)]
struct TypeA {
    value_a: i32,
}

#[derive(SimpleObject)]
struct TypeB {
    value_b: i32
}

#[derive(Union)]
enum MyUnion {
    TypeA(TypeA),
    TypeB(TypeB),
}

struct Query;

#[Object]
impl Query {
    async fn all_data(&self) -> Vec<MyUnion> {
        vec![TypeA { value_a: 10 }.into(), TypeB { value_b: 20 }.into()]
    }
}

# tokio::runtime::Runtime::new().unwrap().block_on(async move {
let schema = Schema::build(Query, EmptyMutation, EmptySubscription).data("hello".to_string()).finish();
let res = schema.execute(r#"
{
    allData {
        ... on TypeA {
            valueA
        }
        ... on TypeB {
            valueB
        }
    }
}"#).await.into_result().unwrap().data;
assert_eq!(res, value!({
    "allData": [
        { "valueA": 10 },
        { "valueB": 20 },
    ]
}));
# });
```
