use async_graphql::*;
use futures::{Stream, StreamExt, TryStreamExt};

#[async_std::test]
pub async fn test_subscription() {
    struct QueryRoot;

    #[derive(SimpleObject)]
    struct Event {
        a: i32,
        b: i32,
    }

    #[Object]
    impl QueryRoot {}

    struct SubscriptionRoot;

    #[Subscription]
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
pub async fn test_subscription_with_ctx_data() {
    struct QueryRoot;

    #[Object]
    impl QueryRoot {}

    struct MyObject;

    #[Object]
    impl MyObject {
        async fn value(&self, ctx: &Context<'_>) -> i32 {
            *ctx.data_unchecked::<i32>()
        }
    }

    struct SubscriptionRoot;

    #[Subscription]
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

    #[Object]
    impl QueryRoot {}

    struct SubscriptionRoot;

    struct Token(String);

    #[Subscription]
    impl SubscriptionRoot {
        async fn values(&self, ctx: &Context<'_>) -> Result<impl Stream<Item = i32>> {
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

    #[derive(SimpleObject)]
    struct Event {
        a: i32,
        b: i32,
    }

    #[Object]
    impl QueryRoot {}

    struct SubscriptionRoot;

    #[Subscription]
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

    #[derive(SimpleObject)]
    struct Event {
        a: i32,
        b: i32,
    }

    #[derive(Interface)]
    #[graphql(field(name = "a", type = "&i32"))]
    enum MyInterface {
        Event(Event),
    }

    #[Object]
    impl QueryRoot {}

    struct SubscriptionRoot;

    #[Subscription]
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

    #[derive(SimpleObject)]
    struct Event {
        a: i32,
        b: i32,
    }

    #[derive(Interface)]
    #[graphql(field(name = "a", type = "&i32"))]
    enum MyInterface {
        Event(Event),
    }

    #[Object]
    impl QueryRoot {}

    struct SubscriptionRoot;

    #[Subscription]
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

    #[Object]
    impl Event {
        async fn value(&self) -> Result<i32> {
            if self.value < 5 {
                Ok(self.value)
            } else {
                Err("TestError".into())
            }
        }
    }

    #[Object]
    impl QueryRoot {}

    struct SubscriptionRoot;

    #[Subscription]
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
        Some(Err(vec![ServerError {
            message: "TestError".to_string(),
            locations: vec![Pos {
                line: 1,
                column: 25
            }],
            path: vec![
                PathSegment::Field("events".to_owned()),
                PathSegment::Field("value".to_owned())
            ],
            extensions: None,
        }]))
    );

    assert!(stream.next().await.is_none());
}

#[async_std::test]
pub async fn test_subscription_fieldresult() {
    struct QueryRoot;

    #[Object]
    impl QueryRoot {}

    struct SubscriptionRoot;

    #[Subscription]
    impl SubscriptionRoot {
        async fn values(&self) -> impl Stream<Item = Result<i32>> {
            futures::stream::iter(0..5)
                .map(Result::Ok)
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
        Some(Err(vec![ServerError {
            message: "StreamErr".to_string(),
            locations: vec![Pos {
                line: 1,
                column: 16
            }],
            path: vec![PathSegment::Field("values".to_owned())],
            extensions: None,
        }]))
    );

    assert!(stream.next().await.is_none());
}
