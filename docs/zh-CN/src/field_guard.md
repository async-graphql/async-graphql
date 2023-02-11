# 字段守卫 (Field Guard)

你可以为`Object`, `SimpleObject`, `ComplexObject`和`Subscription`的字段定义`守卫`，它将在调用字段的 Resolver 函数前执行，如果失败则返回一个错误。

```rust
# extern crate async_graphql;
# use async_graphql::*;
#[derive(Eq, PartialEq, Copy, Clone)]
enum Role {
    Admin,
    Guest,
}

struct RoleGuard {
    role: Role,
}

impl RoleGuard {
    fn new(role: Role) -> Self {
        Self { role }
    }
}

#[async_trait::async_trait]
impl Guard for RoleGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        if ctx.data_opt::<Role>() == Some(&self.role) {
            Ok(())
        } else {
            Err("Forbidden".into())
        }
    }
}
```

用`guard`属性使用它：

```rust
# extern crate async_graphql;
# use async_graphql::*;
# #[derive(Eq, PartialEq, Copy, Clone)]
# enum Role { Admin, Guest, }
# struct RoleGuard { role: Role, }
# impl RoleGuard { fn new(role: Role) -> Self { Self { role } } }
# #[async_trait::async_trait]
# impl Guard for RoleGuard { async fn check(&self, ctx: &Context<'_>) -> Result<()> { todo!() } }
#[derive(SimpleObject)]
struct Query {
    /// 只允许 Admin 访问
    #[graphql(guard = "RoleGuard::new(Role::Admin)")]
    value1: i32,
    /// 允许 Admin 或者 Guest 访问
    #[graphql(guard = "RoleGuard::new(Role::Admin).or(RoleGuard::new(Role::Guest))")]
    value2: i32,
}
```

## 从参数中获取值

有时候守卫需要从字段参数中获取值，你需要像下面这样在创建守卫时传递该参数值：

```rust
# extern crate async_graphql;
# use async_graphql::*;
struct EqGuard {
    expect: i32,
    actual: i32,
}

impl EqGuard {
    fn new(expect: i32, actual: i32) -> Self {
        Self { expect, actual }
    }
}

#[async_trait::async_trait]
impl Guard for EqGuard {
    async fn check(&self, _ctx: &Context<'_>) -> Result<()> {
        if self.expect != self.actual {
            Err("Forbidden".into())
        } else {
            Ok(())
        }
    }
}

struct Query;

#[Object]
impl Query {
    #[graphql(guard = "EqGuard::new(100, value)")]
    async fn get(&self, value: i32) -> i32 {
        value
    }
}
```