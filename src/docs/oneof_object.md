Define a GraphQL oneof input object

# Macro attributes

| Attribute     | description                                                                                                                                                                      | Type         | Optional |
|---------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------------|----------|
| name          | Object name                                                                                                                                                                      | string       | Y        |
| rename_fields | Rename all the fields according to the given case convention. The possible values are "lowercase", "UPPERCASE", "PascalCase", "camelCase", "snake_case", "SCREAMING_SNAKE_CASE". | string       | Y        |
| visible       | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).*                                  | bool         | Y        |
| visible       | Call the specified function. If the return value is `false`, it will not be displayed in introspection.                                                                          | string       | Y        |
| concretes     | Specify how the concrete type of the generic SimpleObject should be implemented.                                                                                                 | ConcreteType | Y        |

# Field attributes

| Attribute    | description                                                                                                                                     | Type        | Optional |
|--------------|-------------------------------------------------------------------------------------------------------------------------------------------------|-------------|----------|
| name         | Field name                                                                                                                                      | string      | Y        |
| validator    | Input value validator *[See also the Book](https://async-graphql.github.io/async-graphql/en/input_value_validators.html)*                       | object      | Y        |
| visible      | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool        | Y        |
| visible      | Call the specified function. If the return value is `false`, it will not be displayed in introspection.                                         | string      | Y        |
| secret       | Mark this field as a secret, it will not output the actual value in the log.                                                                    | bool        | Y        |

# Examples

```rust
use async_graphql::*;

#[derive(OneofObject)]
enum MyInputObject {
    A(i32),
    B(String),
}

struct Query;

#[Object]
impl Query {
    async fn value(&self, input: MyInputObject) -> String {
        match input {
            MyInputObject::A(value) => format!("a:{}", value),
            MyInputObject::B(value) => format!("b:{}", value),
        }
    }
}

# tokio::runtime::Runtime::new().unwrap().block_on(async move {
let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
let res = schema.execute(r#"
{
    value1: value(input:{a:100})
    value2: value(input:{b:"abc"})
}"#).await.into_result().unwrap().data;
assert_eq!(res, value!({ "value1": "a:100", "value2": "b:abc" }));
# });
```
