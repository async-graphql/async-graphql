use async_graphql::*;
use futures_util::stream::{Stream, StreamExt};

#[derive(Eq, PartialEq, Copy, Clone)]
enum Role {
    Admin,
    Guest,
}

pub struct RoleGuard {
    role: Role,
}

impl RoleGuard {
    fn new(role: Role) -> Self {
        Self { role }
    }
}

#[cfg_attr(feature = "boxed-trait", async_trait::async_trait)]
impl Guard for RoleGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        if ctx.data_opt::<Role>() == Some(&self.role) {
            Ok(())
        } else {
            Err("Forbidden".into())
        }
    }
}

struct Username(String);

struct UserGuard<'a> {
    username: &'a str,
}

impl<'a> UserGuard<'a> {
    fn new(username: &'a str) -> Self {
        Self { username }
    }
}

#[cfg_attr(feature = "boxed-trait", async_trait::async_trait)]
impl Guard for UserGuard<'_> {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        if ctx.data_opt::<Username>().map(|name| name.0.as_str()) == Some(self.username) {
            Ok(())
        } else {
            Err("Forbidden".into())
        }
    }
}

#[tokio::test]
pub async fn test_guard_simple_rule() {
    #[derive(SimpleObject)]
    struct Query {
        #[graphql(guard = "RoleGuard::new(Role::Admin)")]
        value: i32,
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        #[graphql(guard = "RoleGuard::new(Role::Admin)")]
        async fn values(&self) -> impl Stream<Item = i32> {
            futures_util::stream::iter(vec![1, 2, 3])
        }
    }

    let schema = Schema::new(Query { value: 10 }, EmptyMutation, Subscription);

    let query = "{ value }";
    assert_eq!(
        schema
            .execute(Request::new(query).data(Role::Admin))
            .await
            .data,
        value!({"value": 10})
    );

    let query = "{ value }";
    assert_eq!(
        schema
            .execute(Request::new(query).data(Role::Guest))
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: "Forbidden".to_string(),
            source: None,
            locations: vec![Pos { line: 1, column: 3 }],
            path: vec![PathSegment::Field("value".to_owned())],
            extensions: None,
        }]
    );

    assert_eq!(
        schema
            .execute_stream(Request::new("subscription { values }").data(Role::Admin))
            .map(|item| item.data)
            .collect::<Vec<_>>()
            .await,
        vec![
            value! ({"values": 1}),
            value! ({"values": 2}),
            value! ({"values": 3})
        ]
    );

    assert_eq!(
        schema
            .execute_stream(Request::new("subscription { values }").data(Role::Guest))
            .next()
            .await
            .unwrap()
            .errors,
        vec![ServerError {
            message: "Forbidden".to_string(),
            source: None,
            locations: vec![Pos {
                line: 1,
                column: 16
            }],
            path: vec![PathSegment::Field("values".to_owned())],
            extensions: None,
        }]
    );
}

#[tokio::test]
pub async fn test_guard_and_operator() {
    #[derive(SimpleObject)]
    struct Query {
        #[graphql(guard = r#"RoleGuard::new(Role::Admin).and(UserGuard::new("test"))"#)]
        value: i32,
    }

    let schema = Schema::new(Query { value: 10 }, EmptyMutation, EmptySubscription);

    let query = "{ value }";
    assert_eq!(
        schema
            .execute(
                Request::new(query)
                    .data(Role::Admin)
                    .data(Username("test".to_string()))
            )
            .await
            .data,
        value!({"value": 10})
    );

    let query = "{ value }";
    assert_eq!(
        schema
            .execute(
                Request::new(query)
                    .data(Role::Guest)
                    .data(Username("test".to_string()))
            )
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: "Forbidden".to_string(),
            source: None,
            locations: vec![Pos { line: 1, column: 3 }],
            path: vec![PathSegment::Field("value".to_owned())],
            extensions: None,
        }]
    );

    let query = "{ value }";
    assert_eq!(
        schema
            .execute(
                Request::new(query)
                    .data(Role::Admin)
                    .data(Username("test1".to_string()))
            )
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: "Forbidden".to_string(),
            source: None,
            locations: vec![Pos { line: 1, column: 3 }],
            path: vec![PathSegment::Field("value".to_owned())],
            extensions: None,
        }]
    );

    let query = "{ value }";
    assert_eq!(
        schema
            .execute(
                Request::new(query)
                    .data(Role::Guest)
                    .data(Username("test1".to_string()))
            )
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: "Forbidden".to_string(),
            source: None,
            locations: vec![Pos { line: 1, column: 3 }],
            path: vec![PathSegment::Field("value".to_owned())],
            extensions: None,
        }]
    );
}

#[tokio::test]
pub async fn test_guard_or_operator() {
    #[derive(SimpleObject)]
    struct Query {
        #[graphql(guard = r#"RoleGuard::new(Role::Admin).or(UserGuard::new("test"))"#)]
        value: i32,
    }

    let schema = Schema::new(Query { value: 10 }, EmptyMutation, EmptySubscription);

    let query = "{ value }";
    assert_eq!(
        schema
            .execute(
                Request::new(query)
                    .data(Role::Admin)
                    .data(Username("test".to_string()))
            )
            .await
            .data,
        value!({"value": 10})
    );

    let query = "{ value }";
    assert_eq!(
        schema
            .execute(
                Request::new(query)
                    .data(Role::Guest)
                    .data(Username("test".to_string()))
            )
            .await
            .data,
        value!({"value": 10})
    );

    let query = "{ value }";
    assert_eq!(
        schema
            .execute(
                Request::new(query)
                    .data(Role::Admin)
                    .data(Username("test1".to_string()))
            )
            .await
            .data,
        value!({"value": 10})
    );

    let query = "{ value }";
    assert_eq!(
        schema
            .execute(
                Request::new(query)
                    .data(Role::Guest)
                    .data(Username("test1".to_string()))
            )
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: "Forbidden".to_string(),
            source: None,
            locations: vec![Pos { line: 1, column: 3 }],
            path: vec![PathSegment::Field("value".to_owned())],
            extensions: None,
        }]
    );
}

#[tokio::test]
pub async fn test_guard_use_params() {
    struct EqGuard {
        expect: i32,
        actual: i32,
    }

    impl EqGuard {
        fn new(expect: i32, actual: i32) -> Self {
            Self { expect, actual }
        }
    }

    #[cfg_attr(feature = "boxed-trait", async_trait::async_trait)]
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

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    assert_eq!(
        schema
            .execute(Request::new("{ get(value: 100) }"))
            .await
            .into_result()
            .unwrap()
            .data,
        value!({"get": 100})
    );

    assert_eq!(
        schema
            .execute(Request::new("{ get(value: 99) }"))
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: "Forbidden".to_string(),
            source: None,
            locations: vec![Pos { line: 1, column: 3 }],
            path: vec![PathSegment::Field("get".to_owned())],
            extensions: None,
        }]
    );
}

#[tokio::test]
pub async fn test_guard_on_simple_object() {
    #[derive(SimpleObject)]
    #[graphql(guard = "RoleGuard::new(Role::Admin)")]
    struct Query {
        value: i32,
    }

    let schema = Schema::new(Query { value: 100 }, EmptyMutation, EmptySubscription);

    let query = "{ value }";
    assert_eq!(
        schema
            .execute(Request::new(query).data(Role::Admin))
            .await
            .data,
        value!({"value": 100})
    );

    let query = "{ value }";
    assert_eq!(
        schema
            .execute(Request::new(query).data(Role::Guest))
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: "Forbidden".to_string(),
            source: None,
            locations: vec![Pos { line: 1, column: 3 }],
            path: vec![PathSegment::Field("value".to_owned())],
            extensions: None,
        }]
    );
}

#[tokio::test]
pub async fn test_guard_on_simple_object_field() {
    #[derive(SimpleObject)]
    #[graphql]
    struct Query {
        #[graphql(guard = "RoleGuard::new(Role::Admin)")]
        value: i32,
    }

    let schema = Schema::new(Query { value: 100 }, EmptyMutation, EmptySubscription);

    let query = "{ value }";
    assert_eq!(
        schema
            .execute(Request::new(query).data(Role::Admin))
            .await
            .data,
        value!({"value": 100})
    );

    let query = "{ value }";
    assert_eq!(
        schema
            .execute(Request::new(query).data(Role::Guest))
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: "Forbidden".to_string(),
            source: None,
            locations: vec![Pos { line: 1, column: 3 }],
            path: vec![PathSegment::Field("value".to_owned())],
            extensions: None,
        }]
    );
}

#[tokio::test]
pub async fn test_guard_on_object() {
    struct Query;

    #[Object(guard = "RoleGuard::new(Role::Admin)")]
    impl Query {
        async fn value(&self) -> i32 {
            100
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    let query = "{ value }";
    assert_eq!(
        schema
            .execute(Request::new(query).data(Role::Admin))
            .await
            .data,
        value!({"value": 100})
    );

    let query = "{ value }";
    assert_eq!(
        schema
            .execute(Request::new(query).data(Role::Guest))
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: "Forbidden".to_string(),
            source: None,
            locations: vec![Pos { line: 1, column: 3 }],
            path: vec![PathSegment::Field("value".to_owned())],
            extensions: None,
        }]
    );
}

#[tokio::test]
pub async fn test_guard_on_object_field() {
    struct Query;

    #[Object]
    impl Query {
        #[graphql(guard = "RoleGuard::new(Role::Admin)")]
        async fn value(&self) -> i32 {
            100
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    let query = "{ value }";
    assert_eq!(
        schema
            .execute(Request::new(query).data(Role::Admin))
            .await
            .data,
        value!({"value": 100})
    );

    let query = "{ value }";
    assert_eq!(
        schema
            .execute(Request::new(query).data(Role::Guest))
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: "Forbidden".to_string(),
            source: None,
            locations: vec![Pos { line: 1, column: 3 }],
            path: vec![PathSegment::Field("value".to_owned())],
            extensions: None,
        }]
    );
}

#[tokio::test]
pub async fn test_guard_on_complex_object() {
    #[derive(SimpleObject)]
    #[graphql(complex)]
    struct Query {
        value1: i32,
    }

    #[ComplexObject(guard = "RoleGuard::new(Role::Admin)")]
    impl Query {
        async fn value2(&self) -> i32 {
            100
        }
    }

    let schema = Schema::new(Query { value1: 10 }, EmptyMutation, EmptySubscription);

    let query = "{ value2 }";
    assert_eq!(
        schema
            .execute(Request::new(query).data(Role::Admin))
            .await
            .data,
        value!({"value2": 100})
    );

    let query = "{ value2 }";
    assert_eq!(
        schema
            .execute(Request::new(query).data(Role::Guest))
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: "Forbidden".to_string(),
            source: None,
            locations: vec![Pos { line: 1, column: 3 }],
            path: vec![PathSegment::Field("value2".to_owned())],
            extensions: None,
        }]
    );
}

#[tokio::test]
pub async fn test_guard_on_complex_object_field() {
    #[derive(SimpleObject)]
    #[graphql(complex)]
    struct Query {
        value1: i32,
    }

    #[ComplexObject]
    impl Query {
        #[graphql(guard = "RoleGuard::new(Role::Admin)")]
        async fn value2(&self) -> i32 {
            100
        }
    }

    let schema = Schema::new(Query { value1: 10 }, EmptyMutation, EmptySubscription);

    let query = "{ value2 }";
    assert_eq!(
        schema
            .execute(Request::new(query).data(Role::Admin))
            .await
            .data,
        value!({"value2": 100})
    );

    let query = "{ value2 }";
    assert_eq!(
        schema
            .execute(Request::new(query).data(Role::Guest))
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: "Forbidden".to_string(),
            source: None,
            locations: vec![Pos { line: 1, column: 3 }],
            path: vec![PathSegment::Field("value2".to_owned())],
            extensions: None,
        }]
    );
}

#[tokio::test]
pub async fn test_guard_with_fn() {
    fn is_admin(ctx: &Context<'_>) -> Result<()> {
        if ctx.data_opt::<Role>() == Some(&Role::Admin) {
            Ok(())
        } else {
            Err("Forbidden".into())
        }
    }

    #[derive(SimpleObject)]
    struct Query {
        #[graphql(guard = "is_admin")]
        value: i32,
    }

    let schema = Schema::new(Query { value: 10 }, EmptyMutation, EmptySubscription);

    let query = "{ value }";
    assert_eq!(
        schema
            .execute(Request::new(query).data(Role::Admin))
            .await
            .data,
        value!({"value": 10})
    );

    assert_eq!(
        schema
            .execute(Request::new(query).data(Role::Guest))
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: "Forbidden".to_string(),
            source: None,
            locations: vec![Pos { line: 1, column: 3 }],
            path: vec![PathSegment::Field("value".to_owned())],
            extensions: None,
        }]
    );
}

#[tokio::test]
pub async fn test_guard_on_nullable_field_simple_object() {
    #[derive(SimpleObject)]
    struct Author {
        id: String,
        #[graphql(guard = "RoleGuard::new(Role::Admin)")]
        age: Option<i32>,
    }

    #[derive(SimpleObject)]
    struct Query {
        post_id: String,
        author: Author,
    }

    let schema = Schema::new(
        Query {
            post_id: "1".to_string(),
            author: Author {
                id: "2".to_string(),
                age: Some(42),
            },
        },
        EmptyMutation,
        EmptySubscription,
    );

    // Admin can see age
    let res = schema
        .execute(Request::new("{ postId author { id age } }").data(Role::Admin))
        .await;
    assert_eq!(
        res.data,
        value!({"postId": "1", "author": {"id": "2", "age": 42}})
    );

    // Guest: age should be null, but the rest of the data should be intact
    let res = schema
        .execute(Request::new("{ postId author { id age } }").data(Role::Guest))
        .await;
    assert_eq!(
        res.data,
        value!({"postId": "1", "author": {"id": "2", "age": null}})
    );
    assert_eq!(res.errors.len(), 1);
    assert_eq!(res.errors[0].message, "Forbidden");
    assert_eq!(
        res.errors[0].path,
        vec![
            PathSegment::Field("author".to_owned()),
            PathSegment::Field("age".to_owned()),
        ]
    );
}

#[tokio::test]
pub async fn test_guard_on_nullable_field_object() {
    struct Query;

    #[Object]
    impl Query {
        async fn author(&self) -> Author {
            Author {
                id: "2".to_string(),
                age: Some(42),
            }
        }
    }

    struct Author {
        id: String,
        age: Option<i32>,
    }

    #[Object]
    impl Author {
        async fn id(&self) -> &str {
            &self.id
        }

        #[graphql(guard = "RoleGuard::new(Role::Admin)")]
        async fn age(&self) -> Option<i32> {
            self.age
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    // Admin can see age
    let res = schema
        .execute(Request::new("{ author { id age } }").data(Role::Admin))
        .await;
    assert_eq!(res.data, value!({"author": {"id": "2", "age": 42}}));

    // Guest: age should be null, rest of data intact
    let res = schema
        .execute(Request::new("{ author { id age } }").data(Role::Guest))
        .await;
    assert_eq!(res.data, value!({"author": {"id": "2", "age": null}}));
    assert_eq!(res.errors.len(), 1);
    assert_eq!(res.errors[0].message, "Forbidden");
}

#[tokio::test]
pub async fn test_guard_on_nullable_field_complex_object() {
    #[derive(SimpleObject)]
    #[graphql(complex)]
    struct Author {
        id: String,
    }

    #[ComplexObject]
    impl Author {
        #[graphql(guard = "RoleGuard::new(Role::Admin)")]
        async fn age(&self) -> Option<i32> {
            Some(42)
        }
    }

    struct Query;

    #[Object]
    impl Query {
        async fn author(&self) -> Author {
            Author {
                id: "2".to_string(),
            }
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    // Admin can see age
    let res = schema
        .execute(Request::new("{ author { id age } }").data(Role::Admin))
        .await;
    assert_eq!(res.data, value!({"author": {"id": "2", "age": 42}}));

    // Guest: age should be null, rest of data intact
    let res = schema
        .execute(Request::new("{ author { id age } }").data(Role::Guest))
        .await;
    assert_eq!(res.data, value!({"author": {"id": "2", "age": null}}));
    assert_eq!(res.errors.len(), 1);
    assert_eq!(res.errors[0].message, "Forbidden");
}

#[tokio::test]
pub async fn test_guard_on_non_nullable_field_still_propagates() {
    #[derive(SimpleObject)]
    struct Query {
        #[graphql(guard = "RoleGuard::new(Role::Admin)")]
        value: i32,
        other: String,
    }

    let schema = Schema::new(
        Query {
            value: 10,
            other: "hello".to_string(),
        },
        EmptyMutation,
        EmptySubscription,
    );

    // Non-nullable field guard failure should still propagate
    let res = schema
        .execute(Request::new("{ value other }").data(Role::Guest))
        .await;
    assert_eq!(res.data, value!(null));
    assert_eq!(res.errors.len(), 1);
    assert_eq!(res.errors[0].message, "Forbidden");
}
