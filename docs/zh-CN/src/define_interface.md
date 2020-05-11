# 接口(Interface)

接口用于抽象具有特定字段集合的对象，`Async-graphql`内部实现实际上是一个包装器，包装器转发接口上定义的Resolve函数到实现该接口的对象，所以接口类型所包含的字段类型，参数都必须和实现该接口的对象完全匹配。

`Async-graphql`自动实现了对象到接口的转换，把一个对象类型转换为接口类型只需要调用`Into::into`。

```rust
use async_graphql::*;

struct Circle {
    radius: f32,
}

#[GqlObject]
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

#[GqlObject]
impl Square {
    async fn area(&self) -> f32 {
        self.width * self.width
    }

    async fn scale(&self, s: f32) -> Shape {
        Square { width: self.width * s }.into()
    }
}

#[GqlInterface(
    field(name = "area", type = "f32"),
    field(name = "scale", type = "Shape", arg(name = "s", type = "f32"))
)]
enum Shape {
    Circle(Circle),
    Square(Square),
}
```
