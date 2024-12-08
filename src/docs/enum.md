Define a GraphQL enum

*[See also the Book](https://async-graphql.github.io/async-graphql/en/define_enum.html).*

# Macro attributes

| Attribute    | description                                                                                                                                                                      | Type   | Optional |
|--------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------|----------|
| name         | Enum name                                                                                                                                                                        | string | Y        |
| name_type    | If `true`, the enum name will be specified from [`async_graphql::TypeName`](https://docs.rs/async-graphql/latest/async_graphql/trait.TypeName.html) trait                        | bool   | Y        |
| display      | Implements `std::fmt::Display` for the enum type                                                                                                                                 | bool   | Y        |
| rename_items | Rename all the fields according to the given case convention. The possible values are "lowercase", "UPPERCASE", "PascalCase", "camelCase", "snake_case", "SCREAMING_SNAKE_CASE". | string | Y        |
| remote       | Derive a remote enum                                                                                                                                                             | string | Y        |
| visible      | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).*                                  | bool   | Y        |
| visible      | Call the specified function. If the return value is `false`, it will not be displayed in introspection.                                                                          | string | Y        |
| inaccessible | Indicate that an enum is not accessible from a supergraph when using Apollo Federation                                                                                           | bool   | Y        |
| tag          | Arbitrary string metadata that will be propagated to the supergraph when using Apollo Federation. This attribute is repeatable                                                   | string | Y        |
| directives   | Directives                                                                                                                                                                       | expr   | Y        |

# Item attributes

| Attribute    | description                                                                                                                                     | Type   | Optional |
|--------------|-------------------------------------------------------------------------------------------------------------------------------------------------|--------|----------|
| name         | Item name                                                                                                                                       | string | Y        |
| deprecation  | Item deprecated                                                                                                                                 | bool   | Y        |
| deprecation  | Item deprecation reason                                                                                                                         | string | Y        |
| visible      | If `false`, it will not be displayed in introspection. *[See also the Book](https://async-graphql.github.io/async-graphql/en/visibility.html).* | bool   | Y        |
| visible      | Call the specified function. If the return value is `false`, it will not be displayed in introspection.                                         | string | Y        |
| inaccessible | Indicate that an item is not accessible from a supergraph when using Apollo Federation                                                          | bool   | Y        |
| tag          | Arbitrary string metadata that will be propagated to the supergraph when using Apollo Federation. This attribute is repeatable                  | string | Y        |
| directives   | Directives                                                                                                                                      | expr   | Y        |

# Examples

```rust
use async_graphql::*;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
enum MyEnum {
    A,
    #[graphql(name = "b")] B,
}

struct Query {
    value1: MyEnum,
    value2: MyEnum,
}

#[Object]
impl Query {
    /// value1
    async fn value1(&self) -> MyEnum {
        self.value1
    }

    /// value2
    async fn value2(&self) -> MyEnum {
        self.value2
    }
}

# tokio::runtime::Runtime::new().unwrap().block_on(async move {
let schema = Schema::new(Query{ value1: MyEnum::A, value2: MyEnum::B }, EmptyMutation, EmptySubscription);
let res = schema.execute("{ value1 value2 }").await.into_result().unwrap().data;
assert_eq!(res, value!({ "value1": "A", "value2": "b" }));
# });
```
