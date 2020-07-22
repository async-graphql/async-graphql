use async_graphql::guard::Guard;
use async_graphql::*;
use futures::{Stream, StreamExt};
use std::sync::Arc;

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
    async fn check(&self, ctx: &Context<'_>) -> FieldResult<()> {
        if ctx.data_opt::<Role>() == Some(&self.role) {
            Ok(())
        } else {
            Err("Forbidden".into())
        }
    }
}

struct Username(String);

struct UserGuard {
    username: String,
}

#[async_trait::async_trait]
impl Guard for UserGuard {
    async fn check(&self, ctx: &Context<'_>) -> FieldResult<()> {
        if ctx.data_opt::<Username>().map(|name| &name.0).as_deref() == Some(&self.username) {
            Ok(())
        } else {
            Err("Forbidden".into())
        }
    }
}

#[async_std::test]
pub async fn test_guard() {
    #[SimpleObject]
    struct MyObj {
        #[field(guard(RoleGuard(role = "Role::Admin")))]
        value: i32,
    }

    struct Query;

    #[Object]
    impl Query {
        #[field(guard(RoleGuard(role = "Role::Admin")))]
        async fn value(&self) -> i32 {
            1
        }

        async fn obj(&self) -> MyObj {
            MyObj { value: 99 }
        }
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        #[field(guard(RoleGuard(role = "Role::Admin")))]
        async fn values(&self) -> impl Stream<Item = i32> {
            futures::stream::iter(vec![1, 2, 3])
        }
    }

    let schema = Schema::new(Query, EmptyMutation, Subscription);

    let query = "{ obj { value } }";
    assert_eq!(
        QueryBuilder::new_single(query)
            .finish()
            .data(Role::Admin)
            .execute(&schema)
            .await
            .unwrap_single()
            .unwrap()
            .data,
        serde_json::json!({
            "obj": {"value": 99}
        })
    );

    let query = "{ obj { value } }";
    assert_eq!(
        QueryBuilder::new_single(query)
            .finish()
            .data(Role::Guest)
            .execute(&schema)
            .await
            .unwrap_single()
            .unwrap_err(),
        Error::Query {
            pos: Pos { line: 1, column: 9 },
            path: Some(serde_json::json!(["obj", "value"])),
            err: QueryError::FieldError {
                err: "Forbidden".to_string(),
                extended_error: None,
            },
        }
    );

    let query = "{ value }";
    assert_eq!(
        QueryBuilder::new_single(query)
            .finish()
            .data(Role::Admin)
            .execute(&schema)
            .await
            .unwrap_single()
            .unwrap()
            .data,
        serde_json::json!({
            "value": 1,
        })
    );

    let query = "{ value }";
    assert_eq!(
        QueryBuilder::new_single(query)
            .finish()
            .data(Role::Guest)
            .execute(&schema)
            .await
            .unwrap_single()
            .unwrap_err(),
        Error::Query {
            pos: Pos { line: 1, column: 3 },
            path: Some(serde_json::json!(["value"])),
            err: QueryError::FieldError {
                err: "Forbidden".to_string(),
                extended_error: None,
            },
        }
    );

    assert_eq!(
        schema
            .create_subscription_stream(
                "subscription { values }",
                None,
                Variables::default(),
                Some(Arc::new({
                    let mut data = Data::default();
                    data.insert(Role::Admin);
                    data
                })),
            )
            .await
            .unwrap()
            .collect::<Vec<_>>()
            .await,
        vec![
            Ok(serde_json::json! ({"values": 1})),
            Ok(serde_json::json! ({"values": 2})),
            Ok(serde_json::json! ({"values": 3}))
        ]
    );

    assert_eq!(
        schema
            .create_subscription_stream(
                "subscription { values }",
                None,
                Variables::default(),
                Some(Arc::new({
                    let mut data = Data::default();
                    data.insert(Role::Guest);
                    data
                })),
            )
            .await
            .err()
            .unwrap(),
        Error::Query {
            pos: Pos {
                line: 1,
                column: 16
            },
            path: Some(serde_json::json!(["values"])),
            err: QueryError::FieldError {
                err: "Forbidden".to_string(),
                extended_error: None,
            },
        }
    );
}

#[async_std::test]
pub async fn test_multiple_guards() {
    #[SimpleObject]
    struct Query {
        #[field(guard(RoleGuard(role = "Role::Admin"), UserGuard(username = r#""test""#)))]
        value: i32,
    }

    let schema = Schema::new(Query { value: 10 }, EmptyMutation, EmptySubscription);

    let query = "{ value }";
    assert_eq!(
        QueryBuilder::new_single(query)
            .finish()
            .data(Role::Admin)
            .data(Username("test".to_string()))
            .execute(&schema)
            .await
            .unwrap_single()
            .unwrap()
            .data,
        serde_json::json!({"value": 10})
    );

    let query = "{ value }";
    assert_eq!(
        QueryBuilder::new_single(query)
            .finish()
            .data(Role::Guest)
            .data(Username("test".to_string()))
            .execute(&schema)
            .await
            .unwrap_single()
            .unwrap_err(),
        Error::Query {
            pos: Pos { line: 1, column: 3 },
            path: Some(serde_json::json!(["value"])),
            err: QueryError::FieldError {
                err: "Forbidden".to_string(),
                extended_error: None,
            },
        }
    );

    let query = "{ value }";
    assert_eq!(
        QueryBuilder::new_single(query)
            .finish()
            .data(Role::Admin)
            .data(Username("test1".to_string()))
            .execute(&schema)
            .await
            .unwrap_single()
            .unwrap_err(),
        Error::Query {
            pos: Pos { line: 1, column: 3 },
            path: Some(serde_json::json!(["value"])),
            err: QueryError::FieldError {
                err: "Forbidden".to_string(),
                extended_error: None,
            },
        }
    );

    let query = "{ value }";
    assert_eq!(
        QueryBuilder::new_single(query)
            .finish()
            .data(Role::Guest)
            .data(Username("test1".to_string()))
            .execute(&schema)
            .await
            .unwrap_single()
            .unwrap_err(),
        Error::Query {
            pos: Pos { line: 1, column: 3 },
            path: Some(serde_json::json!(["value"])),
            err: QueryError::FieldError {
                err: "Forbidden".to_string(),
                extended_error: None,
            },
        }
    );
}

#[async_std::test]
pub async fn test_guard_forward_arguments() {
    struct UserGuard {
        id: ID,
    }

    #[async_trait::async_trait]
    impl Guard for UserGuard {
        async fn check(&self, ctx: &Context<'_>) -> FieldResult<()> {
            if ctx.data_opt::<ID>() != Some(&self.id) {
                Err("Forbidden".into())
            } else {
                Ok(())
            }
        }
    }

    struct QueryRoot;

    #[Object]
    impl QueryRoot {
        #[field(guard(UserGuard(id = "@id")))]
        async fn user(&self, id: ID) -> ID {
            id
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);

    let query = r#"{ user(id: "abc") }"#;
    assert_eq!(
        QueryBuilder::new_single(query)
            .finish()
            .data(ID::from("abc"))
            .execute(&schema)
            .await
            .unwrap_single()
            .unwrap()
            .data,
        serde_json::json!({"user": "abc"})
    );

    let query = r#"{ user(id: "abc") }"#;
    assert_eq!(
        QueryBuilder::new_single(query)
            .finish()
            .data(ID::from("aaa"))
            .execute(&schema)
            .await
            .unwrap_single()
            .unwrap_err(),
        Error::Query {
            pos: Pos { line: 1, column: 3 },
            path: Some(serde_json::json!(["user"])),
            err: QueryError::FieldError {
                err: "Forbidden".to_string(),
                extended_error: None,
            },
        }
    );
}
