Define a GraphQL subscription

*[See also the Book](https://async-graphql.github.io/async-graphql/en/subscription.html).*

The field function is a synchronization function that performs filtering. When true is returned, the message is pushed to the client.
The second parameter is the type of the field.
Starting with the third parameter is one or more filtering conditions, The filter condition is the parameter of the field.
The filter function should be synchronous.

# Macro attributes

| Attribute            | description                                                                                                                                                                         | Type   | Optional |
|----------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------|----------|
| name                 | Object name                                                                                                                                                                         | string | Y        |
| name_type            | If `true`, the object name will be specified from [`async_graphql::TypeName`](https://docs.rs/async-graphql/latest/async_graphql/trait.TypeName.html) trait                         | bool   | Y        |
| rename_fields        | Rename all the fields according to the given case convention. The possible values are "lowercase", "UPPERCASE", "PascalCase", "camelCase", "snake_case", "SCREAMING_SNAKE_CASE".    | string | Y        |
| rename_args          | Rename all the arguments according to the given case convention. The possible values are "lowercase", "UPPERCASE", "PascalCase", "camelCase", "snake_case", "SCREAMING_SNAKE_CASE". | string | Y        |
| extends              | Add fields to an entity that's defined in another service                                                                                                                           | bool   | Y        |
| visible              | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).*                                     | bool   | Y        |
| visible              | Call the specified function. If the return value is `false`, it will not be displayed in introspection.                                                                             | string | Y        |
| use_type_description | Specifies that the description of the type is on the type declaration. [`Description`]()(derive.Description.html)                                                                   | bool   | Y        |
| guard                | Field of guard *[See also the Book](https://async-graphql.github.io/async-graphql/en/field_guard.html)*                                                                             | string | Y        |
| directives           | Directives                                                                                                                                                                          | expr   | Y        |

# Field attributes

| Attribute   | description                                                                                                                                     | Type   | Optional |
|-------------|-------------------------------------------------------------------------------------------------------------------------------------------------|--------|----------|
| name        | Field name                                                                                                                                      | string | Y        |
| deprecation | Field deprecated                                                                                                                                | bool   | Y        |
| deprecation | Field deprecation reason                                                                                                                        | string | Y        |
| guard       | Field of guard *[See also the Book](https://async-graphql.github.io/async-graphql/en/field_guard.html)*                                         | string | Y        |
| visible     | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool   | Y        |
| visible     | Call the specified function. If the return value is `false`, it will not be displayed in introspection.                                         | string | Y        |
| complexity  | Custom field complexity. *[See also the Book](https://async-graphql.github.io/async-graphql/en/depth_and_complexity.html).*                     | bool   | Y        |
| complexity  | Custom field complexity.                                                                                                                        | string | Y        |
| secret      | Mark this field as a secret, it will not output the actual value in the log.                                                                    | bool   | Y        |
| directives  | Directives                                                                                                                                      | expr   | Y        |

# Field argument attributes

| Attribute    | description                                                                                                                                     | Type        | Optional |
|--------------|-------------------------------------------------------------------------------------------------------------------------------------------------|-------------|----------|
| name         | Argument name                                                                                                                                   | string      | Y        |
| desc         | Argument description                                                                                                                            | string      | Y        |
| deprecation  | Argument deprecation                                                                                                                            | bool        | Y        |
| deprecation  | Argument deprecation reason                                                                                                                     | string      | Y        |
| default      | Use `Default::default` for default value                                                                                                        | none        | Y        |
| default      | Argument default value                                                                                                                          | literal     | Y        |
| default_with | Expression to generate default value                                                                                                            | code string | Y        |
| validator    | Input value validator *[See also the Book](https://async-graphql.github.io/async-graphql/en/input_value_validators.html)*                       | object      | Y        |
| visible      | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool        | Y        |
| visible      | Call the specified function. If the return value is `false`, it will not be displayed in introspection.                                         | string      | Y        |
| process_with | Upon successful parsing, invokes specified function. Its signature must be `fn(&mut T)`.                                                        | code path   | Y        |

# Examples

```rust
use async_graphql::*;
use futures_util::stream::{Stream, StreamExt};

struct Subscription;

#[Subscription]
impl Subscription {
    async fn value(&self, condition: i32) -> impl Stream<Item = i32> {
        // Returns the number from 0 to `condition`.
        futures_util::stream::iter(0..condition)
    }
}
```
