use async_graphql::prelude::*;
use async_graphql::{EmptyMutation, EmptySubscription, Pos, QueryError};

#[async_std::test]
pub async fn test_input_value_custom_error() {
    struct Query;

    #[GqlObject]
    impl Query {
        async fn parse_int(&self, _n: i64) -> bool {
            true
        }
    }

    let schema = GqlSchema::new(Query, EmptyMutation, EmptySubscription);
    let query = r#"{ parseInt(n:"A") }"#;
    assert_eq!(
        schema.execute(&query).await.unwrap_err(),
        GqlError::Query {
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
