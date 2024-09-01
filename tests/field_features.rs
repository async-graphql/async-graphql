#![allow(dead_code)]
#![allow(unexpected_cfgs)]

use async_graphql::*;
use futures_util::stream::{Stream, StreamExt};

#[tokio::test]
pub async fn test_field_features() {
    #[derive(SimpleObject)]
    struct MyObj {
        value: i32,
        #[cfg(feature = "bson")]
        value_bson: i32,
        #[cfg(feature = "abc")]
        value_abc: i32,
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        async fn values(&self) -> impl Stream<Item = i32> {
            futures_util::stream::once(async move { 10 })
        }

        #[cfg(feature = "bson")]
        async fn values_bson(&self) -> impl Stream<Item = i32> {
            futures_util::stream::once(async move { 10 })
        }

        #[cfg(feature = "abc")]
        async fn values_abc(&self) -> Pin<Box<dyn Stream<Item = i32> + Send + 'static>> {
            Box::pin(futures_util::stream::once(async move { 10 }))
        }
    }

    struct Query;

    #[Object]
    impl Query {
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

    let schema = Schema::new(Query, EmptyMutation, Subscription);
    let query = "{ value }";
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "value": 10,
        })
    );

    let query = "{ valueBson }";
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "valueBson": 10,
        })
    );

    let query = "{ valueAbc }";
    assert_eq!(
        schema.execute(query).await.into_result().unwrap_err(),
        vec![ServerError {
            message: r#"Unknown field "valueAbc" on type "Query". Did you mean "value"?"#
                .to_owned(),
            source: None,
            locations: vec![Pos { column: 3, line: 1 }],
            path: Vec::new(),
            extensions: None,
        }]
    );

    let query = "{ obj { value } }";
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "obj": { "value": 10 }
        })
    );

    let query = "{ obj { valueBson } }";
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "obj": { "valueBson": 10 }
        })
    );

    let query = "{ obj { valueAbc } }";
    assert_eq!(
        schema.execute(query).await.into_result().unwrap_err(),
        vec![ServerError {
            message: r#"Unknown field "valueAbc" on type "MyObj". Did you mean "value"?"#
                .to_owned(),
            source: None,
            locations: vec![Pos { column: 9, line: 1 }],
            path: Vec::new(),
            extensions: None,
        }]
    );

    let mut stream = schema.execute_stream("subscription { values }");
    assert_eq!(
        stream
            .next()
            .await
            .map(|resp| resp.into_result().unwrap().data)
            .unwrap(),
        value!({
            "values": 10
        })
    );

    let mut stream = schema.execute_stream("subscription { valuesBson }");
    assert_eq!(
        stream.next().await.map(|resp| resp.data).unwrap(),
        value!({
            "valuesBson": 10
        })
    );

    assert_eq!(
        schema
            .execute_stream("subscription { valuesAbc }")
            .next()
            .await
            .unwrap()
            .errors,
        vec![ServerError {
            message: r#"Unknown field "valuesAbc" on type "Subscription". Did you mean "values", "valuesBson"?"#.to_owned(),
            source: None,
            locations: vec![Pos {
                column: 16,
                line: 1
            }],
            path: Vec::new(),
            extensions: None,
        }]
    );
}
