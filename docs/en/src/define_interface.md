# Interface

`Interface` is used to abstract `Object`s with common fields.
`Async-graphql` implemented it as a wrapper.
The wrapper will forward Resolve to the `Object` that implemented this `Interface`.
Therefore, the `Object`'s fields' type, arguments must match with the `Interface`'s.

`Async-graphql` implemented auto conversion from `Object` to `Interface`, you only need to call `Into::into`.

Interface fields names transforms to camelCase in schema definition.
If you need e.g. snake_cased GraphQL field name, you can use both the `name` and `method` attribute.

- When the `name` and `method` exist together, the `name` is the GraphQL field name and the `method` is the resolver function name.
- When only `name` exists, `name.to_camel_case()` is the GraphQL field name and the `name` is the resolver function name.

```rust
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

    #[field(name = "short_description")]
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

    #[field(name = "short_description")]
    async fn short_description(&self) -> String {
        "Square".to_string()
    }
}

#[Interface(
    field(name = "area", type = "f32"),
    field(name = "scale", type = "Shape", arg(name = "s", type = "f32"))
    field(name = "short_description", method = "short_description", type = "String")
)]
enum Shape {
    Circle(Circle),
    Square(Square),
}
```
