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
    let data = schema
        .execute(
            r#"
            fragment A on QueryRoot {
                value5: value @skip(if: true)
                value6: value @skip(if: false)
            }
            
            query {
                value1: value @skip(if: true)
                value2: value @skip(if: false)
                ... @skip(if: true) {
                    value3: value
                }
                ... @skip(if: false) {
                    value4: value
                }
                ... A
            }
        "#,
        )
        .await
        .into_result()
        .unwrap()
        .data;
    assert_eq!(
        data,
        value!({
            "value2": 10,
            "value4": 10,
            "value6": 10,
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
