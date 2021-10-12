# Field Guard

You can define a `guard` to a field of an `Object`. This permits you to add checks before runing the code logic of the field.
`Guard` are made of rules that you need to define before. A rule is a structure which implements the trait `Guard`.

```rust
#[derive(Eq, PartialEq, Copy, Clone)]
enum Role {
    Admin,
    Guest,
}

struct RoleGuard {
    role: Role,
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

Once you have defined your rule you can use it in the `guard` field attribute.
This attribute supports 4 operators to create complex rules :

- `and` : perform an `and` operation between two rules. (If one rule returns an error the `and` operator will return the error. If both rules return an error it's the first one that will be returned).

- `or` : performs an `or` operation between two rules. (If both rules return an error the error returned is the first one)

- `chain` : takes a set of rules and runs them until one returns an error or it returns `Ok` if all rules pass.

- `race` : takes a set of rules and runs them until one returns `Ok` if they all fail, it returns the last error.

```rust
#[derive(SimpleObject)]
struct Query {
    #[graphql(guard(RoleGuard(role = "Role::Admin")))]
    value: i32,
    #[graphql(guard(and(
        RoleGuard(role = "Role::Admin"),
        UserGuard(username = r#""test""#)
    )))]
    value2: i32,
    #[graphql(guard(or(
        RoleGuard(role = "Role::Admin"),
        UserGuard(username = r#""test""#)
    )))]
    value3: i32,
    #[graphql(guard(chain(
        RoleGuard(role = "Role::Admin"),
        UserGuard(username = r#""test""#),
        AgeGuard(age = r#"21"#)
    )))]
    value4: i32,
    #[graphql(guard(race(
        RoleGuard(role = "Role::Admin"),
        UserGuard(username = r#""test""#),
        AgeGuard(age = r#"21"#)
    )))]
    value5: i32,
}
```
