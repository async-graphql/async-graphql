# Union

The definition of a `Union` is similar to an `Interface`, **but with no fields allowed.**.
The implementation is quite similar for `Async-graphql`; from `Async-graphql`'s perspective, `Union` is a subset of `Interface`.

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

#[derive(Union)]
enum Shape {
    Circle(Circle),
    Square(Square),
}
```

## Flattening nested unions

A restriction in GraphQL is the inability to create a union type out of
other union types. All members must be concrete types. To support nested
unions, we can "flatten" members that are unions, bringing their members up
into the parent union. This is done by applying `#[item(flatten)]` on each
member we want to flatten.

```rust
#[derive(async_graphql::Union)]
pub enum TopLevelUnion {
    A(A),

    // Will fail to compile unless we flatten the union member
    #[item(flatten)]
    B(B),
}

#[derive(async_graphql::SimpleObject)]
pub struct A {
    // ...
}

#[derive(async_graphql::Union)]
pub enum B {
    C(C),
    D(D),
}

#[derive(async_graphql::SimpleObject)]
pub struct C {
    // ...
}

#[derive(async_graphql::SimpleObject)]
pub struct D {
    // ...
}
```

The above example transforms the top-level union into this equivalent:

```rust
#[derive(async_graphql::Union)]
pub enum TopLevelUnion {
    A(A),
    C(C),
    D(D),
}
```
