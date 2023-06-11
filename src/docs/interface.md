Define a GraphQL interface

*[See also the Book](https://async-graphql.github.io/async-graphql/en/define_interface.html).*

# Macro attributes

| Attribute     | description                                                                                                                                                                         | Type           | Optional |
|---------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|----------------|----------|
| name          | Object name                                                                                                                                                                         | string         | Y        |
| name_type     | If `true`, the interface name will be specified from [`async_graphql::TypeName`](https://docs.rs/async-graphql/latest/async_graphql/trait.TypeName.html) trait                      | bool           | Y        |
| rename_fields | Rename all the fields according to the given case convention. The possible values are "lowercase", "UPPERCASE", "PascalCase", "camelCase", "snake_case", "SCREAMING_SNAKE_CASE".    | string         | Y        |
| rename_args   | Rename all the arguments according to the given case convention. The possible values are "lowercase", "UPPERCASE", "PascalCase", "camelCase", "snake_case", "SCREAMING_SNAKE_CASE". | string         | Y        |
| field         | Fields of this Interface                                                                                                                                                            | InterfaceField | N        |
| extends       | Add fields to an entity that's defined in another service                                                                                                                           | bool           | Y        |
| visible       | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).*                                     | bool           | Y        |
| visible       | Call the specified function. If the return value is `false`, it will not be displayed in introspection.                                                                             | string         | Y        |
| inaccessible  | Indicate that an interface is not accessible from a supergraph when using Apollo Federation                                                                                         | bool           | Y        |
| tag           | Arbitrary string metadata that will be propagated to the supergraph when using Apollo Federation. This attribute is repeatable                                                      | string         | Y        |

# Field attributes

| Attribute     | description                                                                                                                                                                                                                              | Type                   | Optional |
|---------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|------------------------|----------|
| name          | Field name                                                                                                                                                                                                                               | string                 | N        |
| ty            | Field type                                                                                                                                                                                                                               | string                 | N        |
| method        | Rust resolver method name. If specified, `name` will not be camelCased in schema definition                                                                                                                                              | string                 | Y        |
| desc          | Field description                                                                                                                                                                                                                        | string                 | Y        |
| deprecation   | Field deprecated                                                                                                                                                                                                                         | bool                   | Y        |
| deprecation   | Field deprecation reason                                                                                                                                                                                                                 | string                 | Y        |
| arg           | Field arguments                                                                                                                                                                                                                          | InterfaceFieldArgument | Y        |
| external      | Mark a field as owned by another service. This allows service A to use fields from service B while also knowing at runtime the types of that field.                                                                                      | bool                   | Y        |
| provides      | Annotate the expected returned fieldset from a field on a base type that is guaranteed to be selectable by the gateway.                                                                                                                  | string                 | Y        |
| requires      | Annotate the required input fieldset from a base type for a resolver. It is used to develop a query plan where the required fields may not be needed by the client, but the service may need additional information from other services. | string                 | Y        |
| override_from | Mark the field as overriding a field currently present on another subgraph. It is used to migrate fields between subgraphs.                                                                                                              | string                 | Y        |
| visible       | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).*                                                                                          | bool                   | Y        |
| visible       | Call the specified function. If the return value is `false`, it will not be displayed in introspection.                                                                                                                                  | string                 | Y        |
| inaccessible  | Indicate that a field is not accessible from a supergraph when using Apollo Federation                                                                                                                                                   | bool                   | Y        |
| tag           | Arbitrary string metadata that will be propagated to the supergraph when using Apollo Federation. This attribute is repeatable                                                                                                           | string                 | Y        |

# Field argument attributes

| Attribute    | description                                                                                                                                     | Type        | Optional |
|--------------|-------------------------------------------------------------------------------------------------------------------------------------------------|-------------|----------|
| name         | Argument name                                                                                                                                   | string      | N        |
| ty           | Argument type                                                                                                                                   | string      | N        |
| desc         | Argument description                                                                                                                            | string      | Y        |
| default      | Use `Default::default` for default value                                                                                                        | none        | Y        |
| default      | Argument default value                                                                                                                          | literal     | Y        |
| default_with | Expression to generate default value                                                                                                            | code string | Y        |
| visible      | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool        | Y        |
| visible      | Call the specified function. If the return value is `false`, it will not be displayed in introspection.                                         | string      | Y        |
| secret       | Mark this field as a secret, it will not output the actual value in the log.                                                                    | bool        | Y        |
| inaccessible | Indicate that an argument is not accessible from a supergraph when using Apollo Federation                                                      | bool        | Y        |
| tag          | Arbitrary string metadata that will be propagated to the supergraph when using Apollo Federation. This attribute is repeatable                  | string      | Y        |


# Define an interface

Define TypeA, TypeB, TypeC... Implement the MyInterface

```ignore
#[derive(Interface)]
enum MyInterface {
    TypeA(TypeA),
    TypeB(TypeB),
    TypeC(TypeC),
    ...
}
```

# Fields

The type, name, and parameter fields of the interface must exactly match the type of the
implementation interface, but Result can be omitted.

```rust
use async_graphql::*;

struct TypeA {
    value: i32,
}

#[Object]
impl TypeA {
    /// Returns data borrowed from the context
    async fn value_a<'a>(&self, ctx: &'a Context<'_>) -> Result<&'a str> {
        Ok(ctx.data::<String>()?.as_str())
    }

    /// Returns data borrowed self
    async fn value_b(&self) -> &i32 {
        &self.value
    }

    /// With parameters
    async fn value_c(&self, a: i32, b: i32) -> i32 {
        a + b
    }

    /// Disabled name transformation, don't forget "method" argument in interface!
    #[graphql(name = "value_d")]
    async fn value_d(&self) -> i32 {
        &self.value + 1
    }
}

#[derive(Interface)]
#[graphql(
    field(name = "value_a", ty = "&'ctx str"),
    field(name = "value_b", ty = "&i32"),
    field(name = "value_c", ty = "i32",
        arg(name = "a", ty = "i32"),
        arg(name = "b", ty = "i32")),
    field(name = "value_d", method = "value_d", ty = "i32"),
)]
enum MyInterface {
    TypeA(TypeA)
}

struct Query;

#[Object]
impl Query {
    async fn type_a(&self) -> MyInterface {
        TypeA { value: 10 }.into()
    }
}

# tokio::runtime::Runtime::new().unwrap().block_on(async move {
let schema = Schema::build(Query, EmptyMutation, EmptySubscription).data("hello".to_string()).finish();
let res = schema.execute(r#"
{
    typeA {
        valueA
        valueB
        valueC(a: 3, b: 2)
        value_d
    }
}"#).await.into_result().unwrap().data;
assert_eq!(res, value!({
    "typeA": {
        "valueA": "hello",
        "valueB": 10,
        "valueC": 5,
        "value_d": 11
    }
}));
# });
```
