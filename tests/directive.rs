use async_graphql::*;

#[tokio::test]
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
        .await;
    assert_eq!(
        resp.data,
        value!({
            "value2": 10,
        })
    );
}

#[tokio::test]
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
        .await;
    assert_eq!(
        resp.data,
        value!({
            "value1": 10,
        })
    );
}

#[tokio::test]
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
        .await;
    assert_eq!(
        resp.data,
        value!({
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
        .await;
    assert_eq!(
        resp.data,
        value!({
            "action1": 10,
        })
    );
}
