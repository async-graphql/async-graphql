use async_graphql::*;
use futures::{Stream, StreamExt};

#[async_std::test]
pub async fn test_subscription() {
    struct QueryRoot;

    #[SimpleObject]
    struct Event {
        #[field]
        a: i32,

        #[field]
        b: i32,
    }

    #[Object]
    impl QueryRoot {}

    struct SubscriptionRoot;

    #[Subscription]
    impl SubscriptionRoot {
        #[field]
        async fn values(&self, start: i32, end: i32) -> impl Stream<Item = i32> {
            futures::stream::iter(start..end)
        }

        #[field]
        async fn events(&self, start: i32, end: i32) -> impl Stream<Item = Event> {
            futures::stream::iter((start..end).map(|n| Event { a: n, b: n * 10 }))
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, SubscriptionRoot);

    {
        let mut stream = schema
            .create_subscription_stream(
                "subscription { values(start: 10, end: 20) }",
                None,
                Default::default(),
            )
            .await
            .unwrap();
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
            .create_subscription_stream(
                "subscription { events(start: 10, end: 20) { a b } }",
                None,
                Default::default(),
            )
            .await
            .unwrap();
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

    #[SimpleObject]
    #[derive(Clone)]
    struct Event1 {
        #[field]
        value: i32,
    }

    #[SimpleObject]
    #[derive(Clone)]
    struct Event2 {
        #[field]
        value: i32,
    }

    #[Object]
    impl QueryRoot {}

    struct SubscriptionRoot;

    #[Subscription]
    impl SubscriptionRoot {
        #[field]
        async fn events1(&self) -> impl Stream<Item = Event1> {
            SimpleBroker::<Event1>::subscribe()
        }

        #[field]
        async fn events2(&self) -> impl Stream<Item = Event2> {
            SimpleBroker::<Event2>::subscribe()
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, SubscriptionRoot);
    let mut stream1 = schema
        .create_subscription_stream(
            "subscription { events1 { value } }",
            None,
            Default::default(),
        )
        .await
        .unwrap();
    let mut stream2 = schema
        .create_subscription_stream(
            "subscription { events2 { value } }",
            None,
            Default::default(),
        )
        .await
        .unwrap();

    SimpleBroker::publish(Event1 { value: 10 });
    SimpleBroker::publish(Event2 { value: 88 });
    SimpleBroker::publish(Event1 { value: 15 });
    SimpleBroker::publish(Event2 { value: 99 });

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
