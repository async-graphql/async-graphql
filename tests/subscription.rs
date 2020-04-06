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
        fn values(&self, start: i32, end: i32) -> impl Stream<Item = i32> {
            futures::stream::iter(start..end)
        }

        #[field]
        fn events(&self, start: i32, end: i32) -> impl Stream<Item = Event> {
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
