use async_graphql::*;
use std::sync::Arc;

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
            source: None,
            locations: vec![Pos {
                line: 1,
                column: 14,
            }],
            path: vec![PathSegment::Field("parseInt".to_owned())],
            extensions: None,
        }],
    );
}

#[tokio::test]
pub async fn test_input_box_str() {
    struct Query;

    #[Object]
    impl Query {
        async fn box_str(&self, s: Box<str>) -> String {
            s.to_string()
        }

        async fn arc_str(&self, s: Arc<str>) -> String {
            s.to_string()
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = r#"{ boxStr(s: "abc") arcStr(s: "def") }"#;
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "boxStr": "abc",
            "arcStr": "def",
        })
    );
}

#[tokio::test]
pub async fn test_input_box_slice() {
    struct Query;

    #[Object]
    impl Query {
        async fn box_slice(&self, s: Box<[i32]>) -> Box<[i32]> {
            s
        }

        async fn arc_slice(&self, s: Arc<[i32]>) -> Arc<[i32]> {
            s
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = r#"{ boxSlice(s: [1, 2, 3]) arcSlice(s: [4, 5, 6]) }"#;
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "boxSlice": [1, 2, 3],
            "arcSlice": [4, 5, 6],
        })
    );
}
