use async_graphql::*;

#[async_std::test]
pub async fn test_fieldresult() {
    struct Query;

    #[Object]
    impl Query {
        async fn error(&self) -> FieldResult<i32> {
            Err("TestError".into())
        }

        async fn opt_error(&self) -> Option<FieldResult<i32>> {
            Some(Err("TestError".into()))
        }

        async fn vec_error(&self) -> Vec<FieldResult<i32>> {
            vec![Ok(1), Err("TestError".into())]
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    assert_eq!(
        schema
            .execute("{ error }")
            .await
            .unwrap_single()
            .unwrap_err(),
        Error::Query {
            pos: Pos { line: 1, column: 3 },
            path: Some(serde_json::json!(["error"])),
            err: QueryError::FieldError {
                err: "TestError".to_string(),
                extended_error: None,
            },
        }
    );

    assert_eq!(
        schema
            .execute("{ optError }")
            .await
            .unwrap_single()
            .unwrap_err(),
        Error::Query {
            pos: Pos { line: 1, column: 3 },
            path: Some(serde_json::json!(["optError"])),
            err: QueryError::FieldError {
                err: "TestError".to_string(),
                extended_error: None,
            },
        }
    );

    assert_eq!(
        schema
            .execute("{ vecError }")
            .await
            .unwrap_single()
            .unwrap_err(),
        Error::Query {
            pos: Pos { line: 1, column: 3 },
            path: Some(serde_json::json!(["vecError", 1])),
            err: QueryError::FieldError {
                err: "TestError".to_string(),
                extended_error: None,
            },
        }
    );
}
