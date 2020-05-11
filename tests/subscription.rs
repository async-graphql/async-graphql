use async_graphql::prelude::*;
use async_graphql::*;
use async_graphql::{EmptyMutation, Pos, QueryError, SimpleBroker};
use futures::{Stream, StreamExt};
use std::sync::Arc;

#[async_std::test]
pub async fn test_subscription() {
    struct QueryRoot;

    #[GqlSimpleObject]
    struct Event {
        a: i32,
        b: i32,
    }

    #[GqlObject]
    impl QueryRoot {}

    struct SubscriptionRoot;

    #[GqlSubscription]
    impl SubscriptionRoot {
        async fn values(&self, start: i32, end: i32) -> impl Stream<Item = i32> {
            futures::stream::iter(start..end)
        }

        async fn events(&self, start: i32, end: i32) -> impl Stream<Item = Event> {
            futures::stream::iter((start..end).map(|n| Event { a: n, b: n * 10 }))
        }
    }

    let schema = GqlSchema::new(QueryRoot, EmptyMutation, SubscriptionRoot);

    {
        let mut stream = schema
            .create_subscription_stream(
                "subscription { values(start: 10, end: 20) }",
                None,
                Default::default(),
                None,
            )
            .await
            .unwrap();
        for i in 10..20 {
            assert_eq!(
                Some(Ok(serde_json::json!({ "values": i }))),
                stream.next().await
            );
        }
        assert!(stream.next().await.is_none());
    }

    {
        let mut stream = schema
            .create_subscription_stream(
                "subscription { events(start: 10, end: 20) { a b } }",
                None,
                Default::default(),
                None,
            )
            .await
            .unwrap();
        for i in 10..20 {
            assert_eq!(
                Some(Ok(serde_json::json!({ "events": {"a": i, "b": i * 10} }))),
                stream.next().await
            );
        }
        assert!(stream.next().await.is_none());
    }
}

#[async_std::test]
pub async fn test_simple_broker() {
    struct QueryRoot;

    #[GqlSimpleObject]
    #[derive(Clone)]
    struct Event1 {
        value: i32,
    }

    #[GqlSimpleObject]
    #[derive(Clone)]
    struct Event2 {
        value: i32,
    }

    #[GqlObject]
    impl QueryRoot {}

    struct SubscriptionRoot;

    #[GqlSubscription]
    impl SubscriptionRoot {
        async fn events1(&self) -> impl Stream<Item = Event1> {
            SimpleBroker::<Event1>::subscribe()
        }

        async fn events2(&self) -> impl Stream<Item = Event2> {
            SimpleBroker::<Event2>::subscribe()
        }
    }

    let schema = GqlSchema::new(QueryRoot, EmptyMutation, SubscriptionRoot);
    let mut stream1 = schema
        .create_subscription_stream(
            "subscription { events1 { value } }",
            None,
            Default::default(),
            None,
        )
        .await
        .unwrap();
    let mut stream2 = schema
        .create_subscription_stream(
            "subscription { events2 { value } }",
            None,
            Default::default(),
            None,
        )
        .await
        .unwrap();

    SimpleBroker::publish(Event1 { value: 10 });
    SimpleBroker::publish(Event2 { value: 88 });
    SimpleBroker::publish(Event1 { value: 15 });
    SimpleBroker::publish(Event2 { value: 99 });

    assert_eq!(
        stream1.next().await,
        Some(Ok(serde_json::json!({ "events1": {"value": 10} })))
    );
    assert_eq!(
        stream1.next().await,
        Some(Ok(serde_json::json!({ "events1": {"value": 15} })))
    );

    assert_eq!(
        stream2.next().await,
        Some(Ok(serde_json::json!({ "events2": {"value": 88} })))
    );
    assert_eq!(
        stream2.next().await,
        Some(Ok(serde_json::json!({ "events2": {"value": 99} })))
    );
}

#[async_std::test]
pub async fn test_subscription_with_ctx_data() {
    struct QueryRoot;

    #[GqlObject]
    impl QueryRoot {}

    struct MyObject;

    #[GqlObject]
    impl MyObject {
        async fn value(&self, ctx: &GqlContext<'_>) -> i32 {
            *ctx.data::<i32>()
        }
    }

    struct SubscriptionRoot;

    #[GqlSubscription]
    impl SubscriptionRoot {
        async fn values(&self, ctx: &GqlContext<'_>) -> impl Stream<Item = i32> {
            let value = *ctx.data::<i32>();
            futures::stream::once(async move { value })
        }

        async fn objects(&self) -> impl Stream<Item = MyObject> {
            futures::stream::once(async move { MyObject })
        }
    }

    let schema = GqlSchema::new(QueryRoot, EmptyMutation, SubscriptionRoot);

    {
        let mut stream = schema
            .create_subscription_stream(
                "subscription { values objects { value } }",
                None,
                Default::default(),
                Some(Arc::new({
                    let mut data = GqlData::default();
                    data.insert(100i32);
                    data
                })),
            )
            .await
            .unwrap();
        assert_eq!(
            Some(Ok(serde_json::json!({ "values": 100 }))),
            stream.next().await
        );
        assert_eq!(
            Some(Ok(serde_json::json!({ "objects": { "value": 100 } }))),
            stream.next().await
        );
        assert!(stream.next().await.is_none());
    }
}

#[async_std::test]
pub async fn test_subscription_with_token() {
    struct QueryRoot;

    #[GqlObject]
    impl QueryRoot {}

    struct SubscriptionRoot;

    struct Token(String);

    #[GqlSubscription]
    impl SubscriptionRoot {
        async fn values(&self, ctx: &GqlContext<'_>) -> GqlFieldResult<impl Stream<Item = i32>> {
            if ctx.data::<Token>().0 != "123456" {
                return Err("forbidden".into());
            }
            Ok(futures::stream::once(async move { 100 }))
        }
    }

    let schema = GqlSchema::new(QueryRoot, EmptyMutation, SubscriptionRoot);

    {
        let mut stream = schema
            .create_subscription_stream(
                "subscription { values }",
                None,
                Default::default(),
                Some(Arc::new({
                    let mut data = GqlData::default();
                    data.insert(Token("123456".to_string()));
                    data
                })),
            )
            .await
            .unwrap();
        assert_eq!(
            Some(Ok(serde_json::json!({ "values": 100 }))),
            stream.next().await
        );
        assert!(stream.next().await.is_none());
    }

    {
        assert!(schema
            .create_subscription_stream(
                "subscription { values }",
                None,
                Default::default(),
                Some(Arc::new({
                    let mut data = GqlData::default();
                    data.insert(Token("654321".to_string()));
                    data
                })),
            )
            .await
            .is_err());
    }
}

#[async_std::test]
pub async fn test_subscription_inline_fragment() {
    struct QueryRoot;

    #[GqlSimpleObject]
    struct Event {
        a: i32,
        b: i32,
    }

    #[GqlObject]
    impl QueryRoot {}

    struct SubscriptionRoot;

    #[GqlSubscription]
    impl SubscriptionRoot {
        async fn events(&self, start: i32, end: i32) -> impl Stream<Item = Event> {
            futures::stream::iter((start..end).map(|n| Event { a: n, b: n * 10 }))
        }
    }

    let schema = GqlSchema::new(QueryRoot, EmptyMutation, SubscriptionRoot);
    let mut stream = schema
        .create_subscription_stream(
            r#"
            subscription {
                events(start: 10, end: 20) {
                    a
                    ... {
                        b
                    }
                }
            }
            "#,
            None,
            Default::default(),
            None,
        )
        .await
        .unwrap();
    for i in 10..20 {
        assert_eq!(
            Some(Ok(serde_json::json!({ "events": {"a": i, "b": i * 10} }))),
            stream.next().await
        );
    }
    assert!(stream.next().await.is_none());
}

#[async_std::test]
pub async fn test_subscription_fragment() {
    struct QueryRoot;

    #[GqlSimpleObject]
    struct Event {
        a: i32,
        b: i32,
    }

    #[GqlInterface(field(name = "a", type = "i32"))]
    enum MyInterface {
        Event(Event),
    }

    #[GqlObject]
    impl QueryRoot {}

    struct SubscriptionRoot;

    #[GqlSubscription]
    impl SubscriptionRoot {
        async fn events(&self, start: i32, end: i32) -> impl Stream<Item = Event> {
            futures::stream::iter((start..end).map(|n| Event { a: n, b: n * 10 }))
        }
    }

    let schema = GqlSchema::build(QueryRoot, EmptyMutation, SubscriptionRoot)
        .register_type::<MyInterface>()
        .finish();
    let mut stream = schema
        .create_subscription_stream(
            r#"
            subscription s {
                events(start: 10, end: 20) {
                    ... on MyInterface {
                        a
                    }
                    b
                }
            }
            "#,
            None,
            Default::default(),
            None,
        )
        .await
        .unwrap();
    for i in 10..20 {
        assert_eq!(
            Some(Ok(serde_json::json!({ "events": {"a": i, "b": i * 10} }))),
            stream.next().await
        );
    }
    assert!(stream.next().await.is_none());
}

#[async_std::test]
pub async fn test_subscription_fragment2() {
    struct QueryRoot;

    #[GqlSimpleObject]
    struct Event {
        a: i32,
        b: i32,
    }

    #[GqlInterface(field(name = "a", type = "i32"))]
    enum MyInterface {
        Event(Event),
    }

    #[GqlObject]
    impl QueryRoot {}

    struct SubscriptionRoot;

    #[GqlSubscription]
    impl SubscriptionRoot {
        async fn events(&self, start: i32, end: i32) -> impl Stream<Item = Event> {
            futures::stream::iter((start..end).map(|n| Event { a: n, b: n * 10 }))
        }
    }

    let schema = GqlSchema::build(QueryRoot, EmptyMutation, SubscriptionRoot)
        .register_type::<MyInterface>()
        .finish();
    let mut stream = schema
        .create_subscription_stream(
            r#"
            subscription s {
                events(start: 10, end: 20) {
                    ... Frag
                }
            }

            fragment Frag on Event {
                a b
            }
            "#,
            None,
            Default::default(),
            None,
        )
        .await
        .unwrap();
    for i in 10..20 {
        assert_eq!(
            Some(Ok(serde_json::json!({ "events": {"a": i, "b": i * 10} }))),
            stream.next().await
        );
    }
    assert!(stream.next().await.is_none());
}

#[async_std::test]
pub async fn test_subscription_error() {
    struct QueryRoot;

    struct Event {
        value: i32,
    }

    #[GqlObject]
    impl Event {
        async fn value(&self) -> GqlFieldResult<i32> {
            if self.value < 5 {
                Ok(self.value)
            } else {
                Err("TestError".into())
            }
        }
    }

    #[GqlObject]
    impl QueryRoot {}

    struct SubscriptionRoot;

    #[GqlSubscription]
    impl SubscriptionRoot {
        async fn events(&self) -> impl Stream<Item = Event> {
            futures::stream::iter((0..10).map(|n| Event { value: n }))
        }
    }

    let schema = GqlSchema::new(QueryRoot, EmptyMutation, SubscriptionRoot);
    let mut stream = schema
        .create_subscription_stream(
            "subscription { events { value } }",
            None,
            Default::default(),
            None,
        )
        .await
        .unwrap();
    for i in 0i32..5 {
        assert_eq!(
            Some(Ok(serde_json::json!({ "events": { "value": i } }))),
            stream.next().await
        );
    }
    assert_eq!(
        stream.next().await,
        Some(Err(GqlError::Query {
            pos: Pos {
                line: 1,
                column: 25
            },
            path: Some(serde_json::json!(["events", "value"])),
            err: QueryError::FieldError {
                err: "TestError".to_string(),
                extended_error: None,
            },
        }))
    );

    assert!(stream.next().await.is_none());
}

#[async_std::test]
pub async fn test_subscription_fieldresult() {
    struct QueryRoot;

    #[GqlObject]
    impl QueryRoot {}

    struct SubscriptionRoot;

    #[GqlSubscription]
    impl SubscriptionRoot {
        async fn values(&self) -> impl Stream<Item = GqlFieldResult<i32>> {
            futures::stream::iter(0..5)
                .map(GqlFieldResult::Ok)
                .chain(futures::stream::once(
                    async move { Err("StreamErr".into()) },
                ))
        }
    }

    let schema = GqlSchema::new(QueryRoot, EmptyMutation, SubscriptionRoot);
    let mut stream = schema
        .create_subscription_stream("subscription { values }", None, Default::default(), None)
        .await
        .unwrap();
    for i in 0i32..5 {
        assert_eq!(
            Some(Ok(serde_json::json!({ "values": i }))),
            stream.next().await
        );
    }
    assert_eq!(
        stream.next().await,
        Some(Err(GqlError::Query {
            pos: Pos {
                line: 1,
                column: 16
            },
            path: Some(serde_json::json!(["values"])),
            err: QueryError::FieldError {
                err: "StreamErr".to_string(),
                extended_error: None,
            },
        }))
    );

    assert!(stream.next().await.is_none());
}
