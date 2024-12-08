Define a GraphQL object with fields

*[See also the Book](https://async-graphql.github.io/async-graphql/en/define_simple_object.html).*

Similar to `Object`, but defined on a structure that automatically generates getters for all fields. For a list of valid field types, see [`Object`](attr.Object.html). All fields are converted to camelCase.

# Macro attributes

| Attribute     | description                                                                                                                                                                                             | Type                                       | Optional |
|---------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------------------------------------------|----------|
| name          | Object name                                                                                                                                                                                             | string                                     | Y        |
| name_type     | If `true`, the object name will be specified from [`async_graphql::TypeName`](https://docs.rs/async-graphql/latest/async_graphql/trait.TypeName.html) trait                                             | bool                                       | Y        |
| rename_fields | Rename all the fields according to the given case convention. The possible values are "lowercase", "UPPERCASE", "PascalCase", "camelCase", "snake_case", "SCREAMING_SNAKE_CASE".                        | string                                     | Y        |
| cache_control | Object cache control                                                                                                                                                                                    | [`CacheControl`](struct.CacheControl.html) | Y        |
| extends       | Add fields to an entity that's defined in another service                                                                                                                                               | bool                                       | Y        |
| shareable     | Indicate that an object type's field is allowed to be resolved by multiple subgraphs                                                                                                                    | bool                                       | Y        |
| inaccessible  | Indicate that an object is not accessible from a supergraph when using Apollo Federation                                                                                                                | bool                                       | Y        |
| tag           | Arbitrary string metadata that will be propagated to the supergraph when using Apollo Federation. This attribute is repeatable                                                                          | string                                     | Y        |
| visible       | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).*                                                         | bool                                       | Y        |
| visible       | Call the specified function. If the return value is `false`, it will not be displayed in introspection.                                                                                                 | string                                     | Y        |
| concretes     | Specify how the concrete type of the generic SimpleObject should be implemented. *[See also the Book](https://async-graphql.github.io/async-graphql/en/define_simple_object.html#generic-simpleobjects) | ConcreteType                               | Y        |
| serial        | Resolve each field sequentially.                                                                                                                                                                        | bool                                       | Y        |
| guard         | Field of guard *[See also the Book](https://async-graphql.github.io/async-graphql/en/field_guard.html)*                                                                                                 | string                                     | Y        |
| directives    | Directives                                                                                                                                                                                              | expr                                       | Y        |

# Field attributes

| Attribute     | description                                                                                                                                                                                                                              | Type                                       | Optional |
|---------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------------------------------------------|----------|
| skip          | Skip this field                                                                                                                                                                                                                          | bool                                       | Y        |
| skip_output   | Skip this field, similar to `skip`, but avoids conflicts when this macro is used with `InputObject`.                                                                                                                                     | bool                                       | Y        |
| name          | Field name                                                                                                                                                                                                                               | string                                     | Y        |
| deprecation   | Field deprecated                                                                                                                                                                                                                         | bool                                       | Y        |
| deprecation   | Field deprecation reason                                                                                                                                                                                                                 | string                                     | Y        |
| derived       | Generate derived fields *[See also the Book](https://async-graphql.github.io/async-graphql/en/derived_fields.html).*                                                                                                                     | object                                     | Y        |
| owned         | Field resolver return a ownedship value                                                                                                                                                                                                  | bool                                       | Y        |
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
| flatten       | Similar to serde (flatten)                                                                                                                                                                                                               | boolean                                    | Y        |
| directives    | Directives                                                                                                                                                                                                                               | expr                                       | Y        |
| complexity    | Custom field complexity. *[See also the Book](https://async-graphql.github.io/async-graphql/en/depth_and_complexity.html).*                                                                                                              | bool                                       | Y        |

# Derived attributes

| Attribute | description                                    | Type   | Optional |
|-----------|------------------------------------------------|--------|----------|
| name      | Generated derived field name                   | string | N        |
| into      | Type to derived an into                        | string | Y        |
| owned     | Field resolver return a ownedship value        | bool   | Y        |
| with      | Function to apply to manage advanced use cases | string | Y        |


# Examples

```rust
use async_graphql::*;

#[derive(SimpleObject)]
struct Query {
    value: i32,
}

# tokio::runtime::Runtime::new().unwrap().block_on(async move {
let schema = Schema::new(Query{ value: 10 }, EmptyMutation, EmptySubscription);
let res = schema.execute("{ value }").await.into_result().unwrap().data;
assert_eq!(res, value!({
    "value": 10,
}));
# });
```
