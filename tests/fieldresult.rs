use async_graphql::prelude::*;
use async_graphql::{EmptyMutation, EmptySubscription, Pos, QueryError};

#[async_std::test]
pub async fn test_fieldresult() {
    struct Query;

    #[GqlObject]
    impl Query {
        async fn error(&self) -> GqlFieldResult<i32> {
            Err("TestError".into())
        }

        async fn opt_error(&self) -> Option<GqlFieldResult<i32>> {
            Some(Err("TestError".into()))
        }

        async fn vec_error(&self) -> Vec<GqlFieldResult<i32>> {
            vec![Ok(1), Err("TestError".into())]
        }
    }

    let schema = GqlSchema::new(Query, EmptyMutation, EmptySubscription);

    assert_eq!(
        schema.execute("{ error }").await.unwrap_err(),
        GqlError::Query {
            pos: Pos { line: 1, column: 3 },
            path: Some(serde_json::json!(["error"])),
            err: QueryError::FieldError {
                err: "TestError".to_string(),
                extended_error: None,
            },
        }
    );

    assert_eq!(
        schema.execute("{ optError }").await.unwrap_err(),
        GqlError::Query {
            pos: Pos { line: 1, column: 3 },
            path: Some(serde_json::json!(["optError"])),
            err: QueryError::FieldError {
                err: "TestError".to_string(),
                extended_error: None,
            },
        }
    );

    assert_eq!(
        schema.execute("{ vecError }").await.unwrap_err(),
        GqlError::Query {
            pos: Pos { line: 1, column: 3 },
            path: Some(serde_json::json!(["vecError", 1])),
            err: QueryError::FieldError {
                err: "TestError".to_string(),
                extended_error: None,
            },
        }
    );
}
