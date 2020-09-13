#![allow(dead_code)]

use async_graphql::*;
use futures::{Stream, StreamExt};
use std::pin::Pin;

#[async_std::test]
pub async fn test_field_features() {
    #[derive(GQLSimpleObject)]
    struct MyObj {
        value: i32,

        #[field(feature = "bson")]
        value_bson: i32,

        #[field(feature = "abc")]
        value_abc: i32,
    }

    struct Subscription;

    #[GQLSubscription]
    impl Subscription {
        async fn values(&self) -> impl Stream<Item = i32> {
            futures::stream::once(async move { 10 })
        }

        #[field(feature = "bson")]
        async fn values_bson(&self) -> impl Stream<Item = i32> {
            futures::stream::once(async move { 10 })
        }

        #[field(feature = "abc")]
        async fn values_abc(
            &self,
        ) -> Pin<Box<dyn async_graphql::futures::Stream<Item = i32> + Send + 'static>> {
            Box::pin(futures::stream::once(async move { 10 }))
        }
    }

    struct QueryRoot;

    #[GQLObject]
    impl QueryRoot {
        async fn value(&self) -> i32 {
            10
        }

        #[field(feature = "bson")]
        async fn value_bson(&self) -> i32 {
            10
        }

        #[field(feature = "abc")]
        async fn value_abc(&self) -> i32 {
            10
        }

        async fn obj(&self) -> MyObj {
            MyObj {
                value: 10,
                value_bson: 10,
                value_abc: 10,
            }
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, Subscription);
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
        Error::Query {
            pos: Pos { column: 3, line: 1 },
            path: Some(serde_json::json!(["valueAbc"])),
            err: QueryError::FieldError {
                err: "`valueAbc` is only available if the features `abc` are enabled".to_string(),
                extended_error: None
            }
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
        Error::Query {
            pos: Pos { column: 9, line: 1 },
            path: Some(serde_json::json!(["obj", "valueAbc"])),
            err: QueryError::FieldError {
                err: "`valueAbc` is only available if the features `abc` are enabled".to_string(),
                extended_error: None
            }
        }
    );

    let mut stream = schema.execute_stream("subscription { values }").boxed();
    assert_eq!(
        stream.next().await.map(|resp| resp.into_result().unwrap().data),
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
        Error::Query {
            pos: Pos {
                column: 16,
                line: 1
            },
            path: Some(serde_json::json!(["valuesAbc"])),
            err: QueryError::FieldError {
                err: "`valuesAbc` is only available if the features `abc` are enabled".to_string(),
                extended_error: None
            }
        }
    );
}
