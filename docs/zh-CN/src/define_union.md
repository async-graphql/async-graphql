# 联合(Union)

联合的定义和接口非常像，**但不允许定义字段**。这两个类型的实现原理也差不多，对于`Async-graphql`来说，联合类型是接口类型的子集。

下面把接口定义的例子做一个小小的修改，去掉字段的定义。

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

#[derive(Union)]
enum Shape {
    Circle(Circle),
    Square(Square),
}
```
