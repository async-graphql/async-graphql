use async_graphql::*;

#[tokio::test]
pub async fn test_input_value_custom_error() {
    struct Query;

    #[Object]
    impl Query {
        async fn parse_int(&self, _n: i8) -> bool {
            true
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = r#"{ parseInt(n:289) }"#;
    assert_eq!(
        schema.execute(query).await.into_result().unwrap_err(),
        vec![ServerError {
            message: "Failed to parse \"Int\": Only integers from -128 to 127 are accepted."
                .to_owned(),
            error: None,
            locations: vec![Pos {
                line: 1,
                column: 14,
            }],
            path: vec![PathSegment::Field("parseInt".to_owned())],
            extensions: None,
        }],
    );
}
