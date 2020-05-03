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
        async fn value1(&self) -> FieldResult<i32> {
            Ok(1)
        }

        #[field(guard(RoleGuard(role = "Role::Admin")))]
        async fn value2(&self, ctx1: &Context<'_>) -> FieldResult<i32> {
            Ok(2)
        }

        #[field(guard(RoleGuard(role = "Role::Admin")))]
        async fn value3(&self, _: &Context<'_>) -> i32 {
            3
        }

        async fn obj(&self) -> MyObj {
            MyObj { value: 99 }
        }

        #[entity(guard(RoleGuard(role = "Role::Admin")))]
        async fn find_obj1(&self, value: i32) -> FieldResult<MyObj> {
            Ok(MyObj { value })
        }

        #[entity(guard(RoleGuard(role = "Role::Admin")))]
        #[allow(unused_variables)]
        async fn find_obj2(&self, ctx1: &Context<'_>, value: i32, n: i32) -> FieldResult<MyObj> {
            Ok(MyObj { value })
        }

        #[entity(guard(RoleGuard(role = "Role::Admin")))]
        #[allow(unused_variables)]
        async fn find_obj3(&self, _: &Context<'_>, value: i32, a: i32, b: i32) -> MyObj {
            MyObj { value }
        }
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        #[field(guard(RoleGuard(role = "Role::Admin")))]
        async fn values1(&self) -> FieldResult<impl Stream<Item = i32>> {
            Ok(futures::stream::iter(vec![1, 2, 3]))
        }

        #[field(guard(RoleGuard(role = "Role::Admin")))]
        async fn values2(&self, ctx1: &Context<'_>) -> FieldResult<impl Stream<Item = i32>> {
            Ok(futures::stream::iter(vec![1, 2, 3]))
        }

        #[field(guard(RoleGuard(role = "Role::Admin")))]
        async fn values3(&self, _: &Context<'_>) -> impl Stream<Item = i32> {
            futures::stream::iter(vec![1, 2, 3])
        }
    }

    let schema = Schema::new(Query, EmptyMutation, Subscription);

    let query = "{ obj { value } }";
    assert_eq!(
        QueryBuilder::new(query)
            .data(Role::Admin)
            .execute(&schema)
            .await
            .unwrap()
            .data,
        serde_json::json!({
            "obj": {"value": 99}
        })
    );

    let query = "{ obj { value } }";
    assert_eq!(
        QueryBuilder::new(query)
            .data(Role::Admin)
            .execute(&schema)
            .await
            .unwrap()
            .data,
        serde_json::json!({
            "obj": {"value": 99}
        })
    );

    let query = "{ obj { value } }";
    assert_eq!(
        QueryBuilder::new(query)
            .data(Role::Guest)
            .execute(&schema)
            .await
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

    let query = "{ value1 value2 value3 }";
    assert_eq!(
        QueryBuilder::new(query)
            .data(Role::Admin)
            .execute(&schema)
            .await
            .unwrap()
            .data,
        serde_json::json!({
            "value1": 1,
            "value2": 2,
            "value3": 3,
        })
    );

    let query = "{ value1 }";
    assert_eq!(
        QueryBuilder::new(query)
            .data(Role::Guest)
            .execute(&schema)
            .await
            .unwrap_err(),
        Error::Query {
            pos: Pos { line: 1, column: 3 },
            path: Some(serde_json::json!(["value1"])),
            err: QueryError::FieldError {
                err: "Forbidden".to_string(),
                extended_error: None,
            },
        }
    );

    assert_eq!(
        schema
            .create_subscription_stream(
                "subscription { values1 }",
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
            Ok(serde_json::json! ({"values1": 1})),
            Ok(serde_json::json! ({"values1": 2})),
            Ok(serde_json::json! ({"values1": 3}))
        ]
    );

    assert_eq!(
        schema
            .create_subscription_stream(
                "subscription { values2 }",
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
            Ok(serde_json::json! ({"values2": 1})),
            Ok(serde_json::json! ({"values2": 2})),
            Ok(serde_json::json! ({"values2": 3}))
        ]
    );

    assert_eq!(
        schema
            .create_subscription_stream(
                "subscription { values3 }",
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
            Ok(serde_json::json! ({"values3": 1})),
            Ok(serde_json::json! ({"values3": 2})),
            Ok(serde_json::json! ({"values3": 3}))
        ]
    );

    assert_eq!(
        schema
            .create_subscription_stream(
                "subscription { values1 }",
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
            path: Some(serde_json::json!(["values1"])),
            err: QueryError::FieldError {
                err: "Forbidden".to_string(),
                extended_error: None,
            },
        }
    );

    let query = r#"{
            _entities(representations: [{__typename: "MyObj", value: 1}]) {
                __typename
                ... on MyObj {
                    value
                }
            }
        }"#;
    assert_eq!(
        QueryBuilder::new(query)
            .data(Role::Admin)
            .execute(&schema)
            .await
            .unwrap()
            .data,
        serde_json::json!({
            "_entities": [
                {"__typename": "MyObj", "value": 1},
            ]
        })
    );

    let query = r#"{
            _entities(representations: [{__typename: "MyObj", value: 1, n: 1}]) {
                __typename
                ... on MyObj {
                    value
                }
            }
        }"#;
    assert_eq!(
        QueryBuilder::new(query)
            .data(Role::Admin)
            .execute(&schema)
            .await
            .unwrap()
            .data,
        serde_json::json!({
            "_entities": [
                {"__typename": "MyObj", "value": 1},
            ]
        })
    );

    let query = r#"{
            _entities(representations: [{__typename: "MyObj", value: 1, a: 1, b: 2}]) {
                __typename
                ... on MyObj {
                    value
                }
            }
        }"#;
    assert_eq!(
        QueryBuilder::new(query)
            .data(Role::Admin)
            .execute(&schema)
            .await
            .unwrap()
            .data,
        serde_json::json!({
            "_entities": [
                {"__typename": "MyObj", "value": 1},
            ]
        })
    );

    let query = r#"{
            _entities(representations: [{__typename: "MyObj", value: 1}]) {
                __typename
                ... on MyObj {
                    value
                }
            }
        }"#;
    assert_eq!(
        QueryBuilder::new(query)
            .data(Role::Guest)
            .execute(&schema)
            .await
            .unwrap_err(),
        Error::Query {
            pos: Pos {
                line: 2,
                column: 13
            },
            path: None,
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
        QueryBuilder::new(query)
            .data(Role::Admin)
            .data(Username("test".to_string()))
            .execute(&schema)
            .await
            .unwrap()
            .data,
        serde_json::json!({"value": 10})
    );

    let query = "{ value }";
    assert_eq!(
        QueryBuilder::new(query)
            .data(Role::Guest)
            .data(Username("test".to_string()))
            .execute(&schema)
            .await
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
        QueryBuilder::new(query)
            .data(Role::Admin)
            .data(Username("test1".to_string()))
            .execute(&schema)
            .await
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
        QueryBuilder::new(query)
            .data(Role::Guest)
            .data(Username("test1".to_string()))
            .execute(&schema)
            .await
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
