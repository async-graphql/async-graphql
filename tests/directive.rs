use async_graphql::*;

#[async_std::test]
pub async fn test_directive_skip() {
    struct QueryRoot;

    #[Object]
    impl QueryRoot {
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

#[async_std::test]
pub async fn test_directive_ifdef() {
    struct QueryRoot;

    #[Object]
    impl QueryRoot {
        pub async fn value1(&self) -> i32 {
            10
        }
    }

    struct MutationRoot;

    #[Object]
    impl MutationRoot {
        pub async fn action1(&self) -> i32 {
            10
        }
    }

    let schema = Schema::new(QueryRoot, MutationRoot, EmptySubscription);
    let resp = schema
        .execute(
            r#"
            {
                value1 @ifdef
                value2 @ifdef
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

    let resp = schema
        .execute(
            r#"
            mutation {
                action1 @ifdef
                action2 @ifdef
            }
        "#,
        )
        .await
        .unwrap();
    assert_eq!(
        resp.data,
        serde_json::json!({
            "action1": 10,
        })
    );
}
