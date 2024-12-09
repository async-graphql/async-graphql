Define a GraphQL input object

*[See also the Book](https://async-graphql.github.io/async-graphql/en/define_input_object.html).*

# Macro attributes

| Attribute     | description                                                                                                                                                                      | Type         | Optional |
|---------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------------|----------|
| name          | Object name                                                                                                                                                                      | string       | Y        |
| name_type     | If `true`, the object name will be specified from [`async_graphql::TypeName`](https://docs.rs/async-graphql/latest/async_graphql/trait.TypeName.html) trait                      | bool         | Y        |
| rename_fields | Rename all the fields according to the given case convention. The possible values are "lowercase", "UPPERCASE", "PascalCase", "camelCase", "snake_case", "SCREAMING_SNAKE_CASE". | string       | Y        |
| visible       | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).*                                  | bool         | Y        |
| visible       | Call the specified function. If the return value is `false`, it will not be displayed in introspection.                                                                          | string       | Y        |
| concretes     | Specify how the concrete type of the generic SimpleObject should be implemented.                                                                                                 | ConcreteType | Y        |
| inaccessible  | Indicate that an input object is not accessible from a supergraph when using Apollo Federation                                                                                   | bool         | Y        |
| tag           | Arbitrary string metadata that will be propagated to the supergraph when using Apollo Federation. This attribute is repeatable                                                   | string       | Y        |
| directives    | Directives                                                                                                                                                                       | expr         | Y        |

# Field attributes

| Attribute    | description                                                                                                                                     | Type        | Optional |
|--------------|-------------------------------------------------------------------------------------------------------------------------------------------------|-------------|----------|
| name         | Field name                                                                                                                                      | string      | Y        |
| deprecation  | Field deprecation                                                                                                                               | bool        | Y        |
| deprecation  | Field deprecation reason                                                                                                                        | string      | Y        |
| default      | Use `Default::default` for default value                                                                                                        | none        | Y        |
| default      | Argument default value                                                                                                                          | literal     | Y        |
| default_with | Expression to generate default value                                                                                                            | code string | Y        |
| validator    | Input value validator *[See also the Book](https://async-graphql.github.io/async-graphql/en/input_value_validators.html)*                       | object      | Y        |
| flatten      | Similar to serde (flatten)                                                                                                                      | boolean     | Y        |
| skip         | Skip this field, use `Default::default` to get a default value for this field.                                                                  | bool        | Y        |
| skip_input   | Skip this field, similar to `skip`, but avoids conflicts when this macro is used with `SimpleObject`.                                           | bool        | Y        |
| process_with | Upon successful parsing, invokes specified function. Its signature must be `fn(&mut T)`.                                                        | code path   | Y        |
| visible      | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool        | Y        |
| visible      | Call the specified function. If the return value is `false`, it will not be displayed in introspection.                                         | string      | Y        |
| secret       | Mark this field as a secret, it will not output the actual value in the log.                                                                    | bool        | Y        |
| inaccessible | Indicate that a field is not accessible from a supergraph when using Apollo Federation                                                          | bool        | Y        |
| tag          | Arbitrary string metadata that will be propagated to the supergraph when using Apollo Federation. This attribute is repeatable                  | string      | Y        |
| directives   | Directives                                                                                                                                      | expr        | Y        |

# Examples

```rust
use async_graphql::*;

#[derive(InputObject)]
struct MyInputObject {
    a: i32,
    #[graphql(default = 10)]
    b: i32,
}

struct Query;

#[Object]
impl Query {
    /// value
    async fn value(&self, input: MyInputObject) -> i32 {
        input.a * input.b
    }
}

# tokio::runtime::Runtime::new().unwrap().block_on(async move {
let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
let res = schema.execute(r#"
{
    value1: value(input:{a:9, b:3})
    value2: value(input:{a:9})
}"#).await.into_result().unwrap().data;
assert_eq!(res, value!({ "value1": 27, "value2": 90 }));
# });
```
