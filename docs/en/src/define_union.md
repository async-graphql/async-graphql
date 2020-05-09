# Union

The definition of `Union` is similar to `Interface`'s, **but no field allowed.**.
The implemention is quite similar for `Async-graphql`. 
From `Async-graphql`'s perspective, `Union` is a subset of `Interface`.

The following example modified the definition of `Interface` a little bit and removed fields.

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

#[Union]
struct Shape(Circle, Square);
```