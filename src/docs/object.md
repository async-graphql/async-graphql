Define a GraphQL object with methods

*[See also the Book](https://async-graphql.github.io/async-graphql/en/define_complex_object.html).*

All methods are converted to camelCase.

# Macro attributes

| Attribute            | description                                                                                                                                                                         | Type                                       | Optional |
|----------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------------------------------------------|----------|
| name                 | Object name                                                                                                                                                                         | string                                     | Y        |
| rename_fields        | Rename all the fields according to the given case convention. The possible values are "lowercase", "UPPERCASE", "PascalCase", "camelCase", "snake_case", "SCREAMING_SNAKE_CASE".    | string                                     | Y        |
| rename_args          | Rename all the arguments according to the given case convention. The possible values are "lowercase", "UPPERCASE", "PascalCase", "camelCase", "snake_case", "SCREAMING_SNAKE_CASE". | string                                     | Y        |
| cache_control        | Object cache control                                                                                                                                                                | [`CacheControl`](struct.CacheControl.html) | Y        |
| extends              | Add fields to an entity that's defined in another service                                                                                                                           | bool                                       | Y        |
| shareable            | Indicate that an object type's field is allowed to be resolved by multiple subgraphs                                                                                                | bool                                       | Y        |
| use_type_description | Specifies that the description of the type is on the type declaration. [`Description`]()(derive.Description.html)                                                                   | bool                                       | Y        |
| visible              | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).*                                     | bool                                       | Y        |
| visible              | Call the specified function. If the return value is `false`, it will not be displayed in introspection.                                                                             | string                                     | Y        |
| inaccessible         | Indicate that an object is not accessible from a supergraph when using Apollo Federation                                                                                            | bool                                       | Y        |
| tag                  | Arbitrary string metadata that will be propagated to the supergraph when using Apollo Federation. This attribute is repeatable                                                      | string                                     | Y        |
| serial               | Resolve each field sequentially.                                                                                                                                                    | bool                                       | Y        |
| concretes            | Specify how the concrete type of the generic SimpleObject should be implemented.                                                                                                    | ConcreteType                               | Y        |
| guard                | Field of guard *[See also the Book](https://async-graphql.github.io/async-graphql/en/field_guard.html)*                                                                             | string                                     | Y        |
| directives           | Directives                                                                                                                                                                          | expr                                       | Y        |

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

# Field argument attributes

| Attribute    | description                                                                                                                                     | Type        | Optional |
|--------------|-------------------------------------------------------------------------------------------------------------------------------------------------|-------------|----------|
| name         | Argument name                                                                                                                                   | string      | Y        |
| desc         | Argument description                                                                                                                            | string      | Y        |
| default      | Use `Default::default` for default value                                                                                                        | none        | Y        |
| deprecation  | Argument deprecated                                                                                                                             | bool        | Y        |
| deprecation  | Argument deprecation reason                                                                                                                     | string      | Y        |
| default      | Argument default value                                                                                                                          | literal     | Y        |
| default_with | Expression to generate default value                                                                                                            | code string | Y        |
| validator    | Input value validator *[See also the Book](https://async-graphql.github.io/async-graphql/en/input_value_validators.html)*                       | object      | Y        |
| visible      | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool        | Y        |
| visible      | Call the specified function. If the return value is `false`, it will not be displayed in introspection.                                         | string      | Y        |
| inaccessible | Indicate that an argument is not accessible from a supergraph when using Apollo Federation                                                      | bool        | Y        |
| tag          | Arbitrary string metadata that will be propagated to the supergraph when using Apollo Federation. This attribute is repeatable                  | string      | Y        |
| secret       | Mark this field as a secret, it will not output the actual value in the log.                                                                    | bool        | Y        |
| key          | Is entity key(for Federation)                                                                                                                   | bool        | Y        |
| process_with | Upon successful parsing, invokes specified function. Its signature must be `fn(&mut T)`.                                                        | code path   | Y        |
| directives   | Directives                                                                                                                                      | expr        | Y        |

# Derived argument attributes

| Attribute | description                                    | Type   | Optional |
|-----------|------------------------------------------------|--------|----------|
| name      | Generated derived field name                   | string | N        |
| into      | Type to derived an into                        | string | Y        |
| with      | Function to apply to manage advanced use cases | string | Y        |

# Valid field return types

- Scalar values, such as `i32` and `bool`. `usize`, `isize`, `u128` and `i128` are not supported
- `Vec<T>`, such as `Vec<i32>`
- Slices, such as `&[i32]`
- `Option<T>`, such as `Option<i32>`
- `BTree<T>`, `HashMap<T>`, `HashSet<T>`, `BTreeSet<T>`, `LinkedList<T>`, `VecDeque<T>`
- GraphQL objects.
- GraphQL enums.
- References to any of the above types, such as `&i32` or `&Option<String>`.
- `Result<T, E>`, such as `Result<i32, E>`

# Context

You can define a context as an argument to a method, and the context should be the first argument to the method.

```ignore
#[Object]
impl Query {
    async fn value(&self, ctx: &Context<'_>) -> { ... }
}
```

# Examples

Implements GraphQL Object for struct.

```rust
use async_graphql::*;

struct Query {
    value: i32,
}

#[Object]
impl Query {
    /// value
    async fn value(&self) -> i32 {
        self.value
    }

    /// reference value
    async fn value_ref(&self) -> &i32 {
        &self.value
    }

    /// value with error
    async fn value_with_error(&self) -> Result<i32> {
        Ok(self.value)
    }

    async fn value_with_arg(&self, #[graphql(default = 1)] a: i32) -> i32 {
        a
    }
}

# tokio::runtime::Runtime::new().unwrap().block_on(async move {
let schema = Schema::new(Query { value: 10 }, EmptyMutation, EmptySubscription);
let res = schema.execute(r#"{
    value
    valueRef
    valueWithError
    valueWithArg1: valueWithArg
    valueWithArg2: valueWithArg(a: 99)
}"#).await.into_result().unwrap().data;
assert_eq!(res, value!({
    "value": 10,
    "valueRef": 10,
    "valueWithError": 10,
    "valueWithArg1": 1,
    "valueWithArg2": 99
}));
# });
```

# Examples

Implements GraphQL Object for trait object.

```rust
use async_graphql::*;

trait MyTrait: Send + Sync {
    fn name(&self) -> &str;
}

#[Object]
impl dyn MyTrait + '_ {
    #[graphql(name = "name")]
    async fn gql_name(&self) -> &str {
        self.name()
    }
}

struct MyObj(String);

impl MyTrait for MyObj {
    fn name(&self) -> &str {
        &self.0
    }
}

struct Query;

#[Object]
impl Query {
    async fn objs(&self) -> Vec<Box<dyn MyTrait>> {
        vec![
            Box::new(MyObj("a".to_string())),
            Box::new(MyObj("b".to_string())),
        ]
    }
}

# tokio::runtime::Runtime::new().unwrap().block_on(async move {
let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
let res = schema.execute("{ objs { name } }").await.into_result().unwrap().data;
assert_eq!(res, value!({
    "objs": [
        { "name": "a" },
        { "name": "b" },
    ]
}));
# });
```
