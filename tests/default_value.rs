use async_graphql::*;

#[async_std::test]
pub async fn test_interface_simple_object() {
    struct Query;

    #[Object]
    impl Query {
        async fn value(&self, #[arg(default = "100")] input: i32) -> i32 {
            input
        }
    }

    let query = "{ value }";
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "value": 100
        })
    );

    let query = "{ value(input: 88) }";
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "value": 88
        })
    );
}
