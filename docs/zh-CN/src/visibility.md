# 在内省中隐藏内容

默认情况下，所有类型，字段在内省中都是可见的。但可能你希望根据不同的用户来隐藏一些信息，避免引起不必要的误会。你可以在类型或者字段上添加`visible`属性来做到。

```rust
# extern crate async_graphql;
use async_graphql::*;

#[derive(SimpleObject)]
struct MyObj {
    // 这个字段将在内省中可见
    a: i32,

    // 这个字段在内省中总是隐藏
    #[graphql(visible = false)]
    b: i32, 

    // 这个字段调用 `is_admin` 函数，如果函数的返回值为 `true` 则可见
    #[graphql(visible = "is_admin")]
    c: i32, 
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
enum MyEnum {
    // 这个项目将在内省中可见
    A,

    // 这个项目在内省中总是隐藏
    #[graphql(visible = false)]
    B,

    // 这个项目调用 `is_admin` 函数，如果函数的返回值为 `true` 则可见
    #[graphql(visible = "is_admin")]
    C,
}

struct IsAdmin(bool);

fn is_admin(ctx: &Context<'_>) -> bool {
    ctx.data_unchecked::<IsAdmin>().0
}

```