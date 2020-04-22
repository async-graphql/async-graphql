use async_graphql::*;

#[async_std::test]
pub async fn test_directive_skip() {
    struct QueryRoot;

    #[Object]
    impl QueryRoot {
        #[field]
        pub async fn value(&self) -> i32 {
            10
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
    let resp = schema
        .execute(
            r#"
            {
                value1: value @skip(if: true)
                value2: value @skip(if: false)
            }
        "#,
        )
        .await
        .unwrap();
    assert_eq!(
        resp.data,
        serde_json::json!({
            "value2": 10,
        })
    );
}

#[async_std::test]
pub async fn test_directive_include() {
    struct QueryRoot;

    #[Object]
    impl QueryRoot {
        #[field]
        pub async fn value(&self) -> i32 {
            10
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
    let resp = schema
        .execute(
            r#"
            {
                value1: value @include(if: true)
                value2: value @include(if: false)
            }
        "#,
        )
        .await
        .unwrap();
    assert_eq!(
        resp.data,
        serde_json::json!({
            "value1": 10,
        })
    );
}
