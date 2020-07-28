use async_graphql::*;

#[async_std::test]
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
        schema.execute(&query).await.unwrap_err(),
        Error::Query {
            pos: Pos {
                line: 1,
                column: 14
            },
            path: None,
            err: QueryError::ParseInputValue {
                reason: "Only integers from -128 to 127 are accepted.".to_string()
            },
        }
    );
}
