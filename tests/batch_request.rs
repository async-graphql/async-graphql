use async_graphql::*;

#[async_std::test]
pub async fn test_batch_request() {
    struct Query;

    #[Object]
    impl Query {
        async fn value(&self, a: i32, b: i32) -> i32 {
            a + b
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let batch: BatchRequest = vec![
        Request::new("{ value(a: 10, b: 20) }"),
        Request::new("{ value(a: 30, b: 40) }"),
        Request::new("{ value1 }"),
    ]
    .into();
    let resp = schema.execute_batch(batch).await;
    assert_eq!(
        serde_json::to_value(&resp).unwrap(),
        serde_json::json!([
            {"data": { "value": 30 }},
            {"data": { "value": 70 }},
            {"data": null, "errors": [{
                "message": r#"Unknown field "value1" on type "Query". Did you mean "value"?"#,
                "locations": [{"line": 1, "column": 3}]
            }]},
        ])
    );
}
