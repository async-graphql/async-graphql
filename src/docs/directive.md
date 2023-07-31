Define a directive for query.

*[See also the Book](https://async-graphql.github.io/async-graphql/en/custom_directive.html).*

# Macro attributes

| Attribute   | description                                                                                                                                                                         | Type   | Optional |
|-------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------|----------|
| name        | Object name                                                                                                                                                                         | string | Y        |
| name_type   | If `true`, the directive name will be specified from [`async_graphql::TypeName`](https://docs.rs/async-graphql/latest/async_graphql/trait.TypeName.html) trait                      | bool   | Y        |
| visible     | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).*                                     | bool   | Y        |
| visible     | Call the specified function. If the return value is `false`, it will not be displayed in introspection.                                                                             | string | Y        |
| repeatable  | It means that the directive can be used multiple times in the same location.                                                                                                        | bool   | Y        |
| rename_args | Rename all the arguments according to the given case convention. The possible values are "lowercase", "UPPERCASE", "PascalCase", "camelCase", "snake_case", "SCREAMING_SNAKE_CASE". | string | Y        |
| locations   | Specify the location where the directive is available, multiples are allowed. The possible values is "field", ...                                                                   | string | N        |

# Directive attributes

| Attribute    | description                                                                                                                                     | Type        | Optional |
|--------------|-------------------------------------------------------------------------------------------------------------------------------------------------|-------------|----------|
| name         | Argument name                                                                                                                                   | string      | Y        |
| desc         | Argument description                                                                                                                            | string      | Y        |
| default      | Use `Default::default` for default value                                                                                                        | none        | Y        |
| default      | Argument default value                                                                                                                          | literal     | Y        |
| default_with | Expression to generate default value                                                                                                            | code string | Y        |
| validator    | Input value validator *[See also the Book](https://async-graphql.github.io/async-graphql/en/input_value_validators.html)*                       | object      | Y        |
| visible      | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool        | Y        |
| visible      | Call the specified function. If the return value is `false`, it will not be displayed in introspection.                                         | string      | Y        |
| secret       | Mark this field as a secret, it will not output the actual value in the log.                                                                    | bool        | Y        |

# Examples

```rust
use async_graphql::*;

struct ConcatDirective {
    value: String,
}

#[async_trait::async_trait]
impl CustomDirective for ConcatDirective {
    async fn resolve_field(&self, _ctx: &Context<'_>, resolve: ResolveFut<'_>) -> ServerResult<Option<Value>> {
        resolve.await.map(|value| {
            value.map(|value| match value {
                Value::String(str) => Value::String(str + &self.value),
                _ => value,
            })
        })
    }
}

#[Directive(location = "Field")]
fn concat(value: String) -> impl CustomDirective {
    ConcatDirective { value }
}

struct Query;

#[Object]
impl Query {
    async fn value(&self) -> &'static str {
        "abc"
    }
}

# tokio::runtime::Runtime::new().unwrap().block_on(async move {
let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
    .directive(concat)
    .finish();
let res = schema.execute(r#"{ value @concat(value: "def") }"#).await.into_result().unwrap().data;
assert_eq!(res, value!({
    "value": "abcdef",
}));
# });
```
