# Interface

`Interface` is used to abstract `Object`s with common fields.
`Async-graphql` implemented it as a wrapper.
The wrapper will forward Resolve to the `Object` that implemented this `Interface`.
Therefore, the `Object`'s fields' type, arguments must match with the `Interface`'s.

`Async-graphql` implemented auto conversion from `Object` to `Interface`, you only need to call `Into::into`.

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
}

#[Interface(
    field(name = "area", type = "f32"),
    field(name = "scale", type = "Shape", arg(name = "s", type = "f32"))
)]
enum Shape {
    Circle(Circle),
    Square(Square),
}
```
