use async_graphql::guard::Guard;
use async_graphql::*;
use futures::{Stream, StreamExt};

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

#[async_std::test]
pub async fn test_multiple_guards() {

    #[derive(SimpleObject)]
    struct Query {
        #[graphql(guard(and(RoleGuard(role = "Role::Admin"), UserGuard(username = r#""test""#))))]
        value: i32,
    }

    #[derive(SimpleObject)]
    struct Mutation {
        value_m: i32,
    }


    let schema = Schema::new(Query { value: 10 }, Mutation {valueM: 11}, EmptySubscription);

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
        serde_json::json!({"value": 10})
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
            locations: vec![Pos { line: 1, column: 3 }],
            path: vec![PathSegment::Field("value".to_owned())],
            extensions: None,
        }]
    );

}
