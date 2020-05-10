use async_graphql::*;

#[async_std::test]
pub async fn test_input_value_custom_error() {
    struct Query;

    #[Object]
    impl Query {
        async fn parse_int(&self, _n: i64) -> bool {
            true
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = r#"{ parseInt(n:"A") }"#;
    assert_eq!(
        schema.execute(&query).await.unwrap_err(),
        Error::Query {
            pos: Pos {
                line: 1,
                column: 14
            },
            path: None,
            err: QueryError::ParseInputValue {
                reason: "invalid digit found in string".to_string()
            },
        }
    );
}
