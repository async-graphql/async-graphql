# 接口(Interface)

接口用于抽象具有特定字段集合的对象，`Async-graphql`内部实现实际上是一个包装器，包装器转发接口上定义的Resolver函数到实现该接口的对象，所以接口类型所包含的字段类型，参数都必须和实现该接口的对象完全匹配。

`Async-graphql`自动实现了对象到接口的转换，把一个对象类型转换为接口类型只需要调用`Into::into`。

接口字段的`name`属性表示转发的Resolver函数，并且它将被转换为驼峰命名作为字段名称。
如果你需要自定义GraphQL接口字段名称，可以同时使用`name`和`method`属性。

- 当`name`和`method`属性同时存在时，`name`是GraphQL接口字段名，而`method`是Resolver函数名。
- 当只有`name`存在时, 转换为驼峰命名后的`name`是GraphQL接口字段名，而`name`是Resolver函数名。


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
    field(name = "area", type = "f32"),
    field(name = "scale", type = "Shape", arg(name = "s", type = "f32")),
    field(name = "short_description", method = "short_description", type = "String")
)]
enum Shape {
    Circle(Circle),
    Square(Square),
}
```

## 手工注册接口类型

`Async-graphql`在初始化阶段从`Schema`开始遍历并注册所有被直接或者间接引用的类型，如果某个接口没有被引用到，那么它将不会存在于注册表中，就像下面的例子，
即使`MyObject`实现了`MyInterface`，但由于`Schema`中并没有引用`MyInterface`，类型注册表中将不会存在`MyInterface`类型的信息。

```rust
#[derive(Interface)]
#[graphql(
    field(name = "name", type = "String"),
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

你需要在构造Schema时手工注册`MyInterface`类型：

```rust
Schema::build(Query, EmptyMutation, EmptySubscription)
    .register_type::<MyInterface>()
    .finish();
```
