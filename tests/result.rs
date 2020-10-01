use async_graphql::*;

#[async_std::test]
pub async fn test_fieldresult() {
    struct Query;

    #[Object]
    impl Query {
        async fn error(&self) -> Result<i32> {
            Err("TestError".into())
        }

        async fn opt_error(&self) -> Option<Result<i32>> {
            Some(Err("TestError".into()))
        }

        async fn vec_error(&self) -> Vec<Result<i32>> {
            vec![Ok(1), Err("TestError".into())]
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    assert_eq!(
        schema.execute("{ error }").await.into_result().unwrap_err(),
        vec![ServerError {
            message: "TestError".to_string(),
            locations: vec![Pos { line: 1, column: 3 }],
            path: vec![PathSegment::Field("error".to_owned())],
            extensions: None,
        }]
    );

    assert_eq!(
        schema
            .execute("{ optError }")
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: "TestError".to_string(),
            locations: vec![Pos { line: 1, column: 3 }],
            path: vec![PathSegment::Field("optError".to_owned())],
            extensions: None,
        }]
    );

    assert_eq!(
        schema
            .execute("{ vecError }")
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: "TestError".to_string(),
            locations: vec![Pos { line: 1, column: 3 }],
            path: vec![
                PathSegment::Field("vecError".to_owned()),
                PathSegment::Index(1)
            ],
            extensions: None,
        }]
    );
}
