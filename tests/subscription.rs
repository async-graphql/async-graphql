use async_graphql::*;
use futures::{Stream, StreamExt, TryStreamExt};

#[async_std::test]
pub async fn test_subscription() {
    struct QueryRoot;

    #[derive(GQLSimpleObject)]
    struct Event {
        a: i32,
        b: i32,
    }

    #[GQLObject]
    impl QueryRoot {}

    struct SubscriptionRoot;

    #[GQLSubscription]
    impl SubscriptionRoot {
        async fn values(&self, start: i32, end: i32) -> impl Stream<Item = i32> {
            futures::stream::iter(start..end)
        }

        async fn events(&self, start: i32, end: i32) -> impl Stream<Item = Event> {
            futures::stream::iter((start..end).map(|n| Event { a: n, b: n * 10 }))
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, SubscriptionRoot);

    {
        let mut stream = schema
            .execute_stream("subscription { values(start: 10, end: 20) }")
            .map(|resp| resp.into_result().unwrap().data)
            .boxed();
        for i in 10..20 {
            assert_eq!(
                Some(serde_json::json!({ "values": i })),
                stream.next().await
            );
        }
        assert!(stream.next().await.is_none());
    }

    {
        let mut stream = schema
            .execute_stream("subscription { events(start: 10, end: 20) { a b } }")
            .map(|resp| resp.into_result().unwrap().data)
            .boxed();
        for i in 10..20 {
            assert_eq!(
                Some(serde_json::json!({ "events": {"a": i, "b": i * 10} })),
                stream.next().await
            );
        }
        assert!(stream.next().await.is_none());
    }
}

#[async_std::test]
pub async fn test_simple_broker() {
    struct QueryRoot;

    #[derive(Clone, GQLSimpleObject)]
    struct Event1 {
        value: i32,
    }

    #[derive(Clone, GQLSimpleObject)]
    struct Event2 {
        value: i32,
    }

    #[GQLObject]
    impl QueryRoot {}

    struct SubscriptionRoot;

    #[GQLSubscription]
    impl SubscriptionRoot {
        async fn events1(&self) -> impl Stream<Item = Event1> {
            let stream = SimpleBroker::<Event1>::subscribe();
            SimpleBroker::publish(Event1 { value: 10 });
            SimpleBroker::publish(Event1 { value: 15 });
            stream
        }

        async fn events2(&self) -> impl Stream<Item = Event2> {
            let stream = SimpleBroker::<Event2>::subscribe();
            SimpleBroker::publish(Event2 { value: 88 });
            SimpleBroker::publish(Event2 { value: 99 });
            stream
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, SubscriptionRoot);
    let mut stream1 = schema
        .execute_stream("subscription { events1 { value } }")
        .map(|resp| resp.into_result().unwrap().data)
        .boxed();
    let mut stream2 = schema
        .execute_stream("subscription { events2 { value } }")
        .map(|resp| resp.into_result().unwrap().data)
        .boxed();

    assert_eq!(
        stream1.next().await,
        Some(serde_json::json!({ "events1": {"value": 10} }))
    );
    assert_eq!(
        stream1.next().await,
        Some(serde_json::json!({ "events1": {"value": 15} }))
    );

    assert_eq!(
        stream2.next().await,
        Some(serde_json::json!({ "events2": {"value": 88} }))
    );
    assert_eq!(
        stream2.next().await,
        Some(serde_json::json!({ "events2": {"value": 99} }))
    );
}

#[async_std::test]
pub async fn test_subscription_with_ctx_data() {
    struct QueryRoot;

    #[GQLObject]
    impl QueryRoot {}

    struct MyObject;

    #[GQLObject]
    impl MyObject {
        async fn value(&self, ctx: &Context<'_>) -> i32 {
            *ctx.data_unchecked::<i32>()
        }
    }

    struct SubscriptionRoot;

    #[GQLSubscription]
    impl SubscriptionRoot {
        async fn values(&self, ctx: &Context<'_>) -> impl Stream<Item = i32> {
            let value = *ctx.data_unchecked::<i32>();
            futures::stream::once(async move { value })
        }

        async fn objects(&self) -> impl Stream<Item = MyObject> {
            futures::stream::once(async move { MyObject })
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, SubscriptionRoot);

    {
        let mut stream = schema
            .execute_stream(Request::new("subscription { values objects { value } }").data(100i32))
            .map(|resp| resp.data)
            .boxed();
        assert_eq!(
            Some(serde_json::json!({ "values": 100 })),
            stream.next().await
        );
        assert_eq!(
            Some(serde_json::json!({ "objects": { "value": 100 } })),
            stream.next().await
        );
        assert!(stream.next().await.is_none());
    }
}

#[async_std::test]
pub async fn test_subscription_with_token() {
    struct QueryRoot;

    #[GQLObject]
    impl QueryRoot {}

    struct SubscriptionRoot;

    struct Token(String);

    #[GQLSubscription]
    impl SubscriptionRoot {
        async fn values(&self, ctx: &Context<'_>) -> FieldResult<impl Stream<Item = i32>> {
            if ctx.data_unchecked::<Token>().0 != "123456" {
                return Err("forbidden".into());
            }
            Ok(futures::stream::once(async move { 100 }))
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, SubscriptionRoot);

    {
        let mut stream = schema
            .execute_stream(
                Request::new("subscription { values }").data(Token("123456".to_string())),
            )
            .map(|resp| resp.into_result().unwrap().data)
            .boxed();
        assert_eq!(
            Some(serde_json::json!({ "values": 100 })),
            stream.next().await
        );
        assert!(stream.next().await.is_none());
    }

    {
        assert!(schema
            .execute_stream(
                Request::new("subscription { values }").data(Token("654321".to_string()))
            )
            .boxed()
            .next()
            .await
            .unwrap()
            .is_err());
    }
}

#[async_std::test]
pub async fn test_subscription_inline_fragment() {
    struct QueryRoot;

    #[derive(GQLSimpleObject)]
    struct Event {
        a: i32,
        b: i32,
    }

    #[GQLObject]
    impl QueryRoot {}

    struct SubscriptionRoot;

    #[GQLSubscription]
    impl SubscriptionRoot {
        async fn events(&self, start: i32, end: i32) -> impl Stream<Item = Event> {
            futures::stream::iter((start..end).map(|n| Event { a: n, b: n * 10 }))
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, SubscriptionRoot);
    let mut stream = schema
        .execute_stream(
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
        )
        .map(|resp| resp.data)
        .boxed();
    for i in 10..20 {
        assert_eq!(
            Some(serde_json::json!({ "events": {"a": i, "b": i * 10} })),
            stream.next().await
        );
    }
    assert!(stream.next().await.is_none());
}

#[async_std::test]
pub async fn test_subscription_fragment() {
    struct QueryRoot;

    #[derive(GQLSimpleObject)]
    struct Event {
        a: i32,
        b: i32,
    }

    #[derive(GQLInterface)]
    #[graphql(field(name = "a", type = "&i32"))]
    enum MyInterface {
        Event(Event),
    }

    #[GQLObject]
    impl QueryRoot {}

    struct SubscriptionRoot;

    #[GQLSubscription]
    impl SubscriptionRoot {
        async fn events(&self, start: i32, end: i32) -> impl Stream<Item = Event> {
            futures::stream::iter((start..end).map(|n| Event { a: n, b: n * 10 }))
        }
    }

    let schema = Schema::build(QueryRoot, EmptyMutation, SubscriptionRoot)
        .register_type::<MyInterface>()
        .finish();
    let mut stream = schema
        .execute_stream(
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
        )
        .map(|resp| resp.data)
        .boxed();
    for i in 10i32..20 {
        assert_eq!(
            Some(serde_json::json!({ "events": {"a": i, "b": i * 10} })),
            stream.next().await
        );
    }
    assert!(stream.next().await.is_none());
}

#[async_std::test]
pub async fn test_subscription_fragment2() {
    struct QueryRoot;

    #[derive(GQLSimpleObject)]
    struct Event {
        a: i32,
        b: i32,
    }

    #[derive(GQLInterface)]
    #[graphql(field(name = "a", type = "&i32"))]
    enum MyInterface {
        Event(Event),
    }

    #[GQLObject]
    impl QueryRoot {}

    struct SubscriptionRoot;

    #[GQLSubscription]
    impl SubscriptionRoot {
        async fn events(&self, start: i32, end: i32) -> impl Stream<Item = Event> {
            futures::stream::iter((start..end).map(|n| Event { a: n, b: n * 10 }))
        }
    }

    let schema = Schema::build(QueryRoot, EmptyMutation, SubscriptionRoot)
        .register_type::<MyInterface>()
        .finish();
    let mut stream = schema
        .execute_stream(
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
        )
        .map(|resp| resp.data)
        .boxed();
    for i in 10..20 {
        assert_eq!(
            Some(serde_json::json!({ "events": {"a": i, "b": i * 10} })),
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

    #[GQLObject]
    impl Event {
        async fn value(&self) -> FieldResult<i32> {
            if self.value < 5 {
                Ok(self.value)
            } else {
                Err("TestError".into())
            }
        }
    }

    #[GQLObject]
    impl QueryRoot {}

    struct SubscriptionRoot;

    #[GQLSubscription]
    impl SubscriptionRoot {
        async fn events(&self) -> impl Stream<Item = Event> {
            futures::stream::iter((0..10).map(|n| Event { value: n }))
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, SubscriptionRoot);
    let mut stream = schema
        .execute_stream("subscription { events { value } }")
        .map(|resp| resp.into_result())
        .map_ok(|resp| resp.data)
        .boxed();
    for i in 0i32..5 {
        assert_eq!(
            Some(Ok(serde_json::json!({ "events": { "value": i } }))),
            stream.next().await
        );
    }
    assert_eq!(
        stream.next().await,
        Some(Err(Error::Query {
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

    #[GQLObject]
    impl QueryRoot {}

    struct SubscriptionRoot;

    #[GQLSubscription]
    impl SubscriptionRoot {
        async fn values(&self) -> impl Stream<Item = FieldResult<i32>> {
            futures::stream::iter(0..5)
                .map(FieldResult::Ok)
                .chain(futures::stream::once(
                    async move { Err("StreamErr".into()) },
                ))
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, SubscriptionRoot);
    let mut stream = schema
        .execute_stream("subscription { values }")
        .map(|resp| resp.into_result())
        .map_ok(|resp| resp.data)
        .boxed();
    for i in 0i32..5 {
        assert_eq!(
            Some(Ok(serde_json::json!({ "values": i }))),
            stream.next().await
        );
    }
    assert_eq!(
        stream.next().await,
        Some(Err(Error::Query {
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
