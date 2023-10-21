# Custom directive

There are two types of directives in GraphQL: executable and type system. Executable directives are used by the client within an operation to modify the behavior (like the built-in `@include` and `@skip` directives). Type system directives provide additional information about the types, potentially modifying how the server behaves (like `@deprecated` and `@oneOf`). `async-graphql` allows you to declare both types of custom directives, with different limitations on each. 

## Executable directives

To create a custom executable directive, you need to implement the `CustomDirective` trait, and then use the `Directive` macro to 
generate a factory function that receives the parameters of the directive and returns an instance of the directive.

Currently `async-graphql` only supports custom executable directives located at `FIELD`.

```rust
# extern crate async_graphql;
# use async_graphql::*;
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
```

Register the directive when building the schema:

```rust
# extern crate async_graphql;
# use async_graphql::*;
# struct Query;
# #[Object]
# impl Query { async fn version(&self) -> &str { "1.0" } }
# struct ConcatDirective { value: String, }
# #[async_trait::async_trait]
# impl CustomDirective for ConcatDirective {
#   async fn resolve_field(&self, _ctx: &Context<'_>, resolve: ResolveFut<'_>) -> ServerResult<Option<Value>> { todo!() }
# }
# #[Directive(location = "Field")]
# fn concat(value: String) -> impl CustomDirective { ConcatDirective { value } }
let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
    .directive(concat)
    .finish();
```

## Type system directives

To create a custom type system directive, you can use the `#[TypeDirective]` macro on a function:

```rust
# extern crate async_graphql;
# use async_graphql::*;
#[TypeDirective(
    location = "FieldDefinition",
    location = "Object",
)]
fn testDirective(scope: String, input: u32, opt: Option<u64>) {}
```

Current only the `FieldDefinition` and `Object` locations are supported, you can select one or both. After declaring the directive, you can apply it to a relevant location (after importing the function) like this:

```rust
# extern crate async_graphql;
# use async_graphql::*;
# #[TypeDirective(
# location = "FieldDefinition",
# location = "Object",
# )]
# fn testDirective(scope: String, input: u32, opt: Option<u64>) {}
#[derive(SimpleObject)]
#[graphql(
    directive = testDirective::apply("simple object type".to_string(), 1, Some(3))
)]
struct SimpleValue {
    #[graphql(
        directive = testDirective::apply("field and param with \" symbol".to_string(), 2, Some(3))
    )]
    some_data: String,
}
```

This example produces a schema like this:

```graphql
type SimpleValue @testDirective(scope: "simple object type", input: 1, opt: 3) {
	someData: String! @testDirective(scope: "field and param with \" symbol", input: 2, opt: 3)
}

directive @testDirective(scope: String!, input: Int!, opt: Int) on FIELD_DEFINITION | OBJECT
```

Note: To use a type-system directive with Apollo Federation's `@composeDirective`, see [the federation docs](./apollo_federation#composeDirective)