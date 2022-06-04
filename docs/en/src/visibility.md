# Hide content in introspection

By default, all types and fields are visible in introspection. But maybe you want to hide some content according to different users to avoid unnecessary misunderstandings. You can add the `visible` attribute to the type or field to do it.

```rust
# extern crate async_graphql;
use async_graphql::*;

#[derive(SimpleObject)]
struct MyObj {
    // This field will be visible in introspection.
    a: i32,

    // This field is always hidden in introspection.
    #[graphql(visible = false)]
    b: i32,

    // This field calls the `is_admin` function, which 
    // is visible if the return value is `true`.
    #[graphql(visible = "is_admin")]
    c: i32,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
enum MyEnum {
    // This item will be visible in introspection.
    A,

    // This item is always hidden in introspection.
    #[graphql(visible = false)]
    B,

    // This item calls the `is_admin` function, which 
    // is visible if the return value is `true`.
    #[graphql(visible = "is_admin")]
    C,
}

struct IsAdmin(bool);

fn is_admin(ctx: &Context<'_>) -> bool {
    ctx.data_unchecked::<IsAdmin>().0
}

```
