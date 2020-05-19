use async_graphql::*;

#[async_std::test]
pub async fn test_defer() {
    struct Query {
        value: i32,
    }

    #[Object]
    impl Query {
        async fn value(&self) -> Deferred<i32> {
            10.into()
        }
    }

    let schema = Schema::new(Query { value: 10 }, EmptyMutation, EmptySubscription);
    let query = r#"{
        value
    }"#;
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "value": 10,
        })
    );
}
