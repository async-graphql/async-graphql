#![allow(dead_code)]

use async_graphql::*;
use futures::{Stream, StreamExt};

#[async_std::test]
pub async fn test_field_features() {
    #[derive(SimpleObject)]
    struct MyObj {
        value: i32,
        #[cfg(feature = "bson")]
        value_bson: i32,
        #[cfg(feature = "abc")]
        value_abc: i32,
    }

    struct SubscriptionRoot;

    #[Subscription]
    impl SubscriptionRoot {
        async fn values(&self) -> impl Stream<Item = i32> {
            futures::stream::once(async move { 10 })
        }

        #[cfg(feature = "bson")]
        async fn values_bson(&self) -> impl Stream<Item = i32> {
            futures::stream::once(async move { 10 })
        }

        #[cfg(feature = "abc")]
        async fn values_abc(
            &self,
        ) -> Pin<Box<dyn async_graphql::futures::Stream<Item = i32> + Send + 'static>> {
            Box::pin(futures::stream::once(async move { 10 }))
        }
    }

    struct QueryRoot;

    #[Object]
    impl QueryRoot {
        async fn value(&self) -> i32 {
            10
        }

        #[cfg(feature = "bson")]
        async fn value_bson(&self) -> i32 {
            10
        }

        #[cfg(feature = "abc")]
        async fn value_abc(&self) -> i32 {
            10
        }

        async fn obj(&self) -> MyObj {
            MyObj {
                value: 10,
                #[cfg(feature = "bson")]
                value_bson: 10,
                #[cfg(feature = "abc")]
                value_abc: 10,
            }
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, SubscriptionRoot);
    let query = "{ value }";
    assert_eq!(
        schema.execute(query).await.data,
        serde_json::json!({
            "value": 10,
        })
    );

    let query = "{ valueBson }";
    assert_eq!(
        schema.execute(query).await.data,
        serde_json::json!({
            "valueBson": 10,
        })
    );

    let query = "{ valueAbc }";
    assert_eq!(
        schema.execute(query).await.into_result().unwrap_err(),
        Error::Rule {
            errors: vec![RuleError {
                locations: vec![Pos { line: 1, column: 3 }],
                message: r#"Unknown field "valueAbc" on type "QueryRoot". Did you mean "value"?"#
                    .to_string(),
            }]
            .into(),
        }
    );

    let query = "{ obj { value } }";
    assert_eq!(
        schema.execute(query).await.data,
        serde_json::json!({
            "obj": { "value": 10 }
        })
    );

    let query = "{ obj { valueBson } }";
    assert_eq!(
        schema.execute(query).await.data,
        serde_json::json!({
            "obj": { "valueBson": 10 }
        })
    );

    let query = "{ obj { valueAbc } }";
    assert_eq!(
        schema.execute(query).await.into_result().unwrap_err(),
        Error::Rule {
            errors: vec![RuleError {
                locations: vec![Pos { line: 1, column: 9 }],
                message: r#"Unknown field "valueAbc" on type "MyObj". Did you mean "value"?"#
                    .to_string(),
            }]
            .into(),
        }
    );

    let mut stream = schema.execute_stream("subscription { values }").boxed();
    assert_eq!(
        stream
            .next()
            .await
            .map(|resp| resp.into_result().unwrap().data),
        Some(serde_json::json!({
            "values": 10
        }))
    );

    let mut stream = schema.execute_stream("subscription { valuesBson }").boxed();
    assert_eq!(
        stream.next().await.map(|resp| resp.data),
        Some(serde_json::json!({
            "valuesBson": 10
        }))
    );

    assert_eq!(
        schema
            .execute_stream("subscription { valuesAbc }")
            .boxed()
            .next()
            .await
            .unwrap()
            .error
            .unwrap(),
        Error::Rule {
            errors: vec![RuleError {
                locations: vec![Pos {
                    line: 1,
                    column: 16
                }],
                message:
                    r#"Unknown field "valuesAbc" on type "SubscriptionRoot". Did you mean "values", "valuesBson"?"#
                        .to_string(),
            }]
            .into(),
        }
    );
}
