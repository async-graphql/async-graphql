use async_graphql::*;
use futures::{SinkExt, Stream, StreamExt};

#[async_std::test]
pub async fn test_subscription_ws_transport() {
    struct QueryRoot;

    #[Object]
    impl QueryRoot {}

    struct SubscriptionRoot;

    #[Subscription]
    impl SubscriptionRoot {
        async fn values(&self) -> impl Stream<Item = i32> {
            futures::stream::iter(0..10)
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, SubscriptionRoot);
    let (mut sink, mut stream) = schema.subscription_connection(WebSocketTransport::default());

    sink.send(
        serde_json::to_vec(&serde_json::json!({
            "type": "connection_init",
            "payload": { "token": "123456" }
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(
        Some(serde_json::json!({
        "type": "connection_ack",
        })),
        serde_json::from_slice(&stream.next().await.unwrap()).unwrap()
    );

    sink.send(
        serde_json::to_vec(&serde_json::json!({
            "type": "start",
            "id": "1",
            "payload": {
                "query": "subscription { values }"
            },
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    for i in 0..10 {
        assert_eq!(
            Some(serde_json::json!({
            "type": "data",
            "id": "1",
            "payload": { "data": { "values": i } },
            })),
            serde_json::from_slice(&stream.next().await.unwrap()).unwrap()
        );
    }
}

#[async_std::test]
pub async fn test_subscription_ws_transport_with_token() {
    struct Token(String);

    struct QueryRoot;

    #[Object]
    impl QueryRoot {}

    struct SubscriptionRoot;

    #[Subscription]
    impl SubscriptionRoot {
        async fn values(&self, ctx: &Context<'_>) -> FieldResult<impl Stream<Item = i32>> {
            if ctx.data_unchecked::<Token>().0 != "123456" {
                return Err("forbidden".into());
            }
            Ok(futures::stream::iter(0..10))
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, SubscriptionRoot);

    let (mut sink, mut stream) = schema.subscription_connection(WebSocketTransport::new(|value| {
        #[derive(serde::Deserialize)]
        struct Payload {
            token: String,
        }

        let payload: Payload = serde_json::from_value(value).unwrap();
        let mut data = Data::default();
        data.insert(Token(payload.token));
        Ok(data)
    }));

    sink.send(
        serde_json::to_vec(&serde_json::json!({
            "type": "connection_init",
            "payload": { "token": "123456" }
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(
        Some(serde_json::json!({
        "type": "connection_ack",
        })),
        serde_json::from_slice(&stream.next().await.unwrap()).unwrap()
    );

    sink.send(
        serde_json::to_vec(&serde_json::json!({
            "type": "start",
            "id": "1",
            "payload": {
                "query": "subscription { values }"
            },
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    for i in 0..10 {
        assert_eq!(
            Some(serde_json::json!({
            "type": "data",
            "id": "1",
            "payload": { "data": { "values": i } },
            })),
            serde_json::from_slice(&stream.next().await.unwrap()).unwrap()
        );
    }
}

#[async_std::test]
pub async fn test_subscription_ws_transport_error() {
    struct QueryRoot;

    struct Event {
        value: i32,
    }

    #[Object]
    impl Event {
        async fn value(&self) -> FieldResult<i32> {
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

    let (mut sink, mut stream) =
        schema.subscription_connection(WebSocketTransport::new(|_| Ok(Data::default())));

    sink.send(
        serde_json::to_vec(&serde_json::json!({
            "type": "connection_init"
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(
        Some(serde_json::json!({
        "type": "connection_ack",
        })),
        serde_json::from_slice(&stream.next().await.unwrap()).unwrap()
    );

    sink.send(
        serde_json::to_vec(&serde_json::json!({
            "type": "start",
            "id": "1",
            "payload": {
                "query": "subscription { events { value } }"
            },
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    for i in 0i32..5 {
        assert_eq!(
            Some(serde_json::json!({
            "type": "data",
            "id": "1",
            "payload": { "data": { "events": { "value": i } } },
            })),
            serde_json::from_slice(&stream.next().await.unwrap()).unwrap()
        );
    }

    assert_eq!(
        Some(serde_json::json!({
        "type": "error",
        "id": "1",
        "payload": [{
                "message": "TestError",
                "locations": [{"line": 1, "column": 25}],
                "path": ["events", "value"],
            }],
        })),
        serde_json::from_slice(&stream.next().await.unwrap()).unwrap()
    );
}

#[async_std::test]
pub async fn test_query_over_websocket() {
    struct QueryRoot;

    #[Object]
    impl QueryRoot {
        async fn value(&self) -> i32 {
            999
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
    let (mut sink, mut stream) = schema.subscription_connection(WebSocketTransport::default());

    sink.send(
        serde_json::to_vec(&serde_json::json!({
            "type": "connection_init",
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(
        Some(serde_json::json!({
        "type": "connection_ack",
        })),
        serde_json::from_slice(&stream.next().await.unwrap()).unwrap()
    );

    sink.send(
        serde_json::to_vec(&serde_json::json!({
            "type": "start",
            "id": "1",
            "payload": {
                "query": "query { value }"
            },
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(
        Some(serde_json::json!({
            "type": "data",
            "id": "1",
            "payload": { "data": { "value": 999 } },
        })),
        serde_json::from_slice(&stream.next().await.unwrap()).unwrap()
    );

    assert_eq!(
        Some(serde_json::json!({
            "type": "complete",
            "id": "1",
        })),
        serde_json::from_slice(&stream.next().await.unwrap()).unwrap()
    );
}
