Define a complex GraphQL object for SimpleObject's complex field resolver.

*[See also the Book](https://async-graphql.github.io/async-graphql/en/define_simple_object.html).*

Sometimes most of the fields of a GraphQL object simply return the value of the structure member, but a few
fields are calculated. Usually we use the `Object` macro to define such a GraphQL object.

But this can be done more beautifully with the `ComplexObject` macro. We can use the `SimpleObject` macro to define
some simple fields, and use the `ComplexObject` macro to define some other fields that need to be calculated.

# Macro attributes

| Attribute     | description                                                                                                                                                                         | Type   | Optional |
|---------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------|----------|
| rename_fields | Rename all the fields according to the given case convention. The possible values are "lowercase", "UPPERCASE", "PascalCase", "camelCase", "snake_case", "SCREAMING_SNAKE_CASE".    | string | Y        |
| rename_args   | Rename all the arguments according to the given case convention. The possible values are "lowercase", "UPPERCASE", "PascalCase", "camelCase", "snake_case", "SCREAMING_SNAKE_CASE". | string | Y        |
| guard         | Field of guard *[See also the Book](https://async-graphql.github.io/async-graphql/en/field_guard.html)*                                                                             | string | Y        |
| inaccessible  | Indicate that an object is not accessible from a supergraph when using Apollo Federation                                                                                            | bool   | Y        |
| tag           | Arbitrary string metadata that will be propagated to the supergraph when using Apollo Federation. This attribute is repeatable                                                      | string | Y        |

# Field attributes

| Attribute     | description                                                                                                                                                                                                                              | Type                                       | Optional |
|---------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------------------------------------------|----------|
| skip          | Skip this field                                                                                                                                                                                                                          | bool                                       | Y        |
| name          | Field name                                                                                                                                                                                                                               | string                                     | Y        |
| desc          | Field description                                                                                                                                                                                                                        | string                                     | Y        |
| deprecation   | Field deprecated                                                                                                                                                                                                                         | bool                                       | Y        |
| deprecation   | Field deprecation reason                                                                                                                                                                                                                 | string                                     | Y        |
| cache_control | Field cache control                                                                                                                                                                                                                      | [`CacheControl`](struct.CacheControl.html) | Y        |
| external      | Mark a field as owned by another service. This allows service A to use fields from service B while also knowing at runtime the types of that field.                                                                                      | bool                                       | Y        |
| provides      | Annotate the expected returned fieldset from a field on a base type that is guaranteed to be selectable by the gateway.                                                                                                                  | string                                     | Y        |
| requires      | Annotate the required input fieldset from a base type for a resolver. It is used to develop a query plan where the required fields may not be needed by the client, but the service may need additional information from other services. | string                                     | Y        |
| shareable     | Indicate that a field is allowed to be resolved by multiple subgraphs                                                                                                                                                                    | bool                                       | Y        |
| inaccessible  | Indicate that a field is not accessible from a supergraph when using Apollo Federation                                                                                                                                                   | bool                                       | Y        |
| tag           | Arbitrary string metadata that will be propagated to the supergraph when using Apollo Federation. This attribute is repeatable                                                                                                           | string                                     | Y        |
| override_from | Mark the field as overriding a field currently present on another subgraph. It is used to migrate fields between subgraphs.                                                                                                              | string                                     | Y        |
| guard         | Field of guard *[See also the Book](https://async-graphql.github.io/async-graphql/en/field_guard.html)*                                                                                                                                  | string                                     | Y        |
| visible       | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).*                                                                                          | bool                                       | Y        |
| visible       | Call the specified function. If the return value is `false`, it will not be displayed in introspection.                                                                                                                                  | string                                     | Y        |
| complexity    | Custom field complexity. *[See also the Book](https://async-graphql.github.io/async-graphql/en/depth_and_complexity.html).*                                                                                                              | bool                                       | Y        |
| complexity    | Custom field complexity.                                                                                                                                                                                                                 | string                                     | Y        |
| derived       | Generate derived fields *[See also the Book](https://async-graphql.github.io/async-graphql/en/derived_fields.html).*                                                                                                                     | object                                     | Y        |
| flatten       | Similar to serde (flatten)                                                                                                                                                                                                               | boolean                                    | Y        |
| directives    | Directives                                                                                                                                                                                                                               | expr                                       | Y        |

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
| inaccessible | Indicate that a field argument is not accessible from a supergraph when using Apollo Federation                                                 | bool        | Y        |
| tag          | Arbitrary string metadata that will be propagated to the supergraph when using Apollo Federation. This attribute is repeatable                  | string      | Y        |
| visible      | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool        | Y        |
| visible      | Call the specified function. If the return value is `false`, it will not be displayed in introspection.                                         | string      | Y        |
| secret       | Mark this field as a secret, it will not output the actual value in the log.                                                                    | bool        | Y        |
| process_with | Upon successful parsing, invokes specified function. Its signature must be `fn(&mut T)`.                                                        | code path   | Y        |

# Examples

```rust
use async_graphql::*;

#[derive(SimpleObject)]
#[graphql(complex)] // NOTE: If you want the `ComplexObject` macro to take effect, this `complex` attribute is required.
struct MyObj {
    a: i32,
    b: i32,
}

#[ComplexObject]
impl MyObj {
    async fn c(&self) -> i32 {
        self.a + self.b
    }
}

struct Query;

#[Object]
impl Query {
    async fn obj(&self) -> MyObj {
        MyObj { a: 10, b: 20 }
    }
}

# tokio::runtime::Runtime::new().unwrap().block_on(async move {
let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
let res = schema.execute("{ obj { a b c } }").await.into_result().unwrap().data;
assert_eq!(res, value!({
    "obj": {
        "a": 10,
        "b": 20,
        "c": 30,
    },
}));
# });
```
