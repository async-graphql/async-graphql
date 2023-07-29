# Interface

`Interface` is used to abstract `Object`s with common fields.
`Async-graphql` implements it as a wrapper.
The wrapper will forward field resolution to the `Object` that implements this `Interface`.
Therefore, the `Object`'s fields' type and arguments must match with the `Interface`'s.

`Async-graphql` implements auto conversion from `Object` to `Interface`, you only need to call `Into::into`.

Interface field names are transformed to camelCase for the schema definition.
If you need e.g. a snake_cased GraphQL field name, you can use both the `name` and `method` attributes.

- When `name` and `method` exist together, `name` is the GraphQL field name and the `method` is the resolver function name.
- When only `name` exists, `name.to_camel_case()` is the GraphQL field name and the `name` is the resolver function name.

```rust
# extern crate async_graphql;
use async_graphql::*;

struct Circle {
    radius: f32,
}

#[Object]
impl Circle {
    async fn area(&self) -> f32 {
        std::f32::consts::PI * self.radius * self.radius
    }

    async fn scale(&self, s: f32) -> Shape {
        Circle { radius: self.radius * s }.into()
    }

    #[graphql(name = "short_description")]
    async fn short_description(&self) -> String {
        "Circle".to_string()
    }
}

struct Square {
    width: f32,
}

#[Object]
impl Square {
    async fn area(&self) -> f32 {
        self.width * self.width
    }

    async fn scale(&self, s: f32) -> Shape {
        Square { width: self.width * s }.into()
    }

    #[graphql(name = "short_description")]
    async fn short_description(&self) -> String {
        "Square".to_string()
    }
}

#[derive(Interface)]
#[graphql(
    field(name = "area", ty = "f32"),
    field(name = "scale", ty = "Shape", arg(name = "s", ty = "f32")),
    field(name = "short_description", method = "short_description", ty = "String")
)]
enum Shape {
    Circle(Circle),
    Square(Square),
}
```

## Register the interface manually

`Async-graphql` traverses and registers all directly or indirectly referenced types from `Schema` in the initialization phase.
If an interface is not referenced, it will not exist in the registry, as in the following example , even if `MyObject` implements `MyInterface`,
because `MyInterface` is not referenced in `Schema`, the `MyInterface` type will not exist in the registry.

```rust
# extern crate async_graphql;
# use async_graphql::*;
#[derive(Interface)]
#[graphql(
    field(name = "name", ty = "String"),
)]
enum MyInterface {
    MyObject(MyObject),
}

#[derive(SimpleObject)]
struct MyObject {
    name: String,
}

struct Query;

#[Object]
impl Query {
    async fn obj(&self) -> MyObject {
        todo!()
    }
}

type MySchema = Schema<Query, EmptyMutation, EmptySubscription>;
```

You need to manually register the `MyInterface` type when constructing the `Schema`:

```rust
# extern crate async_graphql;
# use async_graphql::*;
# #[derive(Interface)]
# #[graphql(field(name = "name", ty = "String"))]
# enum MyInterface { MyObject(MyObject) }
# #[derive(SimpleObject)]
# struct MyObject { name: String, }
# struct Query;
# #[Object]
# impl Query { async fn version(&self) -> &str { "1.0" } }

Schema::build(Query, EmptyMutation, EmptySubscription)
    .register_output_type::<MyInterface>()
    .finish();
```
