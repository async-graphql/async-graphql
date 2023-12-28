# 联合 (Union)

联合的定义和接口非常像，**但不允许定义字段**。这两个类型的实现原理也差不多，对于`Async-graphql`来说，联合类型是接口类型的子集。

下面把接口定义的例子做一个小小的修改，去掉字段的定义。

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

## 展平嵌套联合

GraphQL 的有个限制是`Union`类型内不能包含其它联合类型。所有成员必须为`Object`。 
位置支持嵌套`Union`，我们可以用`#graphql(flatten)`，是它们合并到上级`Union`类型。
```rust
# extern crate async_graphql;
#[derive(async_graphql::Union)]
pub enum TopLevelUnion {
    A(A),

    // 除非我们使用 `flatten` 属性，否则将无法编译
    #[graphql(flatten)]
    B(B),
}

#[derive(async_graphql::SimpleObject)]
pub struct A {
    a: i32,
    // ...
}

#[derive(async_graphql::Union)]
pub enum B {
    C(C),
    D(D),
}

#[derive(async_graphql::SimpleObject)]
pub struct C {
    c: i32,
    // ...
}

#[derive(async_graphql::SimpleObject)]
pub struct D {
    d: i32,
    // ...
}
```

上面的示例将顶级`Union`转换为以下等效形式：

```rust
# extern crate async_graphql;
# #[derive(async_graphql::SimpleObject)]
# struct A { a: i32 }
# #[derive(async_graphql::SimpleObject)]
# struct C { c: i32 }
# #[derive(async_graphql::SimpleObject)]
# struct D { d: i32 }
#[derive(async_graphql::Union)]
pub enum TopLevelUnion {
    A(A),
    C(C),
    D(D),
}
```
