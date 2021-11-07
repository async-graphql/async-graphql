use async_graphql::guard::Guard;
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

mod guards {
    pub use super::RoleGuard;
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

struct Username(String);

struct UserGuard {
    username: String,
}

#[async_trait::async_trait]
impl Guard for UserGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        if ctx.data_opt::<Username>().map(|name| &name.0).as_deref() == Some(&self.username) {
            Ok(())
        } else {
            Err("Forbidden".into())
        }
    }
}

struct Age(i32);

struct AgeGuard {
    age: i32,
}

#[async_trait::async_trait]
impl Guard for AgeGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        if ctx.data_opt::<Age>().map(|name| &name.0) == Some(&self.age) {
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
        #[graphql(guard(guards::RoleGuard(role = "Role::Admin")))]
        value: i32,
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        #[graphql(guard(RoleGuard(role = "Role::Admin")))]
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
        #[graphql(guard(and(
            RoleGuard(role = "Role::Admin"),
            UserGuard(username = r#""test""#)
        )))]
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
        #[graphql(guard(or(RoleGuard(role = "Role::Admin"), UserGuard(username = r#""test""#))))]
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
pub async fn test_guard_chain_operator() {
    #[derive(SimpleObject)]
    struct Query {
        #[graphql(guard(chain(
            RoleGuard(role = "Role::Admin"),
            UserGuard(username = r#""test""#),
            AgeGuard(age = r#"21"#)
        )))]
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
                    .data(Age(21))
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
                    .data(Age(21))
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
                    .data(Age(21))
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
                    .data(Username("test".to_string()))
                    .data(Age(22))
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
                    .data(Age(22))
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
pub async fn test_guard_race_operator() {
    #[derive(SimpleObject)]
    struct Query {
        #[graphql(guard(race(
            RoleGuard(role = "Role::Admin"),
            UserGuard(username = r#""test""#),
            AgeGuard(age = r#"21"#)
        )))]
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
                    .data(Age(21))
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
                    .data(Age(22))
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
                    .data(Age(22))
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
                    .data(Age(21))
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
                    .data(Age(22))
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
        #[graphql(guard(EqGuard(expect = "100", actual = "@value")))]
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
