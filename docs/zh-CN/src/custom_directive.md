# 自定义指令

`Async-graphql`可以很方便的自定义指令，这可以扩展 GraphQL 的行为。

创建一个自定义指令，需要实现 `CustomDirective` trait，然后用`Directive`宏生成一个工厂函数，该函数接收指令的参数并返回指令的实例。

目前`Async-graphql`仅支持添加`FIELD`位置的指令。

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

创建模式时注册指令：

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
