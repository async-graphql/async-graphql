# Field Guard

You can define a `guard` for the fields of `Object`, `SimpleObject`, `ComplexObject` and `Subscription`, it will be executed before calling the resolver function, and an error will be returned if it fails.

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

impl Guard for RoleGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        if ctx.data::<Role>().ok() == Some(&self.role) {
            Ok(())
        } else {
            Err("Forbidden".into())
        }
    }
}
```

Use it with the `guard` attribute:

```rust
# extern crate async_graphql;
# use async_graphql::*;
# #[derive(Eq, PartialEq, Copy, Clone)]
# enum Role { Admin, Guest, }
# struct RoleGuard { role: Role, }
# impl RoleGuard { fn new(role: Role) -> Self { Self { role } } }
# impl Guard for RoleGuard { async fn check(&self, ctx: &Context<'_>) -> Result<()> { todo!() } }
#[derive(SimpleObject)]
struct Query {
    /// Only allow Admin
    #[graphql(guard = "RoleGuard::new(Role::Admin)")]
    value1: i32,
    /// Allow Admin or Guest
    #[graphql(guard = "RoleGuard::new(Role::Admin).or(RoleGuard::new(Role::Guest))")]
    value2: i32,
}
```

## Use parameter value

Sometimes guards need to use field parameters, you need to pass the parameter value when creating the guard like this:

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
