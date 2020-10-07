# 字段守卫(Field Guard)

您可以在给`Object`的字段定义`守卫`。 这允许在运行字段的代码逻辑之前添加检查。
义`守卫`由你预先定义的规则组成。 规则是一种实现`Guard`特质的结构。
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

一旦定义了规则，就可以在`guard`字段属性中使用它。
此属性支持4个运算符创建复杂的规则：

-`and`：在两个规则之间执行`与`运算。 （如果一个规则返回错误，则`and`运算符将返回错误。如果两个规则均返回错误，则将是第一个返回的错误）。

-`or`：在两个规则之间执行`或`运算。 （如果两个规则都返回错误，则返回的错误是第一个）

-`chain`：采用一组规则并运行它们，直到返回错误或如果所有规则都通过则返回`Ok`。

-`race`：采用一组规则并运行它们，直到其中一个返回`Ok`。

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

