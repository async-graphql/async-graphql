use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    time::Duration,
};

use async_graphql::{
    http::{WebSocketProtocols, WsMessage},
    *,
};
use futures_channel::mpsc;
use futures_util::{
    stream::{BoxStream, Stream, StreamExt},
    SinkExt,
};

#[tokio::test]
pub async fn test_subscription_ws_transport() {
    struct Query;

    #[Object]
    impl Query {
        async fn value(&self) -> i32 {
            10
        }
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        async fn values(&self) -> impl Stream<Item = i32> {
            futures_util::stream::iter(0..10)
        }
    }

    let schema = Schema::new(Query, EmptyMutation, Subscription);
    let (mut tx, rx) = mpsc::unbounded();
    let mut stream = http::WebSocket::new(schema, rx, WebSocketProtocols::GraphQLWS);

    tx.send(
        serde_json::to_string(&value!({
            "type": "connection_init",
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&stream.next().await.unwrap().unwrap_text())
            .unwrap(),
        serde_json::json!({
            "type": "connection_ack",
        }),
    );

    tx.send(
        serde_json::to_string(&value!({
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
            serde_json::from_str::<serde_json::Value>(&stream.next().await.unwrap().unwrap_text())
                .unwrap(),
            serde_json::json!({
                "type": "next",
                "id": "1",
                "payload": { "data": { "values": i } },
            }),
        );
    }

    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&stream.next().await.unwrap().unwrap_text())
            .unwrap(),
        serde_json::json!({
            "type": "complete",
            "id": "1",
        }),
    );
}

#[tokio::test]
pub async fn test_subscription_ws_transport_with_token() {
    struct Token(String);

    struct Query;

    #[Object]
    impl Query {
        async fn value(&self) -> i32 {
            10
        }
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        async fn values(&self, ctx: &Context<'_>) -> Result<impl Stream<Item = i32>> {
            if ctx.data_unchecked::<Token>().0 != "123456" {
                return Err("forbidden".into());
            }
            Ok(futures_util::stream::iter(0..10))
        }
    }

    let schema = Schema::new(Query, EmptyMutation, Subscription);
    let (mut tx, rx) = mpsc::unbounded();
    let mut stream = http::WebSocket::new(schema.clone(), rx, WebSocketProtocols::GraphQLWS)
        .on_connection_init(|value| async {
            #[derive(serde::Deserialize)]
            struct Payload {
                token: String,
            }

            let payload: Payload = serde_json::from_value(value).unwrap();
            let mut data = Data::default();
            data.insert(Token(payload.token));
            Ok(data)
        });

    tx.send(
        serde_json::to_string(&value!({
            "type": "connection_init",
            "payload": { "token": "123456" }
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(
        Some(value!({
            "type": "connection_ack",
        })),
        serde_json::from_str(&stream.next().await.unwrap().unwrap_text()).unwrap()
    );

    tx.send(
        serde_json::to_string(&value!({
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
            Some(value!({
                "type": "next",
                "id": "1",
                "payload": { "data": { "values": i } },
            })),
            serde_json::from_str(&stream.next().await.unwrap().unwrap_text()).unwrap()
        );
    }

    assert_eq!(
        Some(value!({
            "type": "complete",
            "id": "1",
        })),
        serde_json::from_str(&stream.next().await.unwrap().unwrap_text()).unwrap()
    );

    let (mut tx, rx) = mpsc::unbounded();
    let mut data = Data::default();
    data.insert(Token("123456".to_string()));
    let mut stream =
        http::WebSocket::new(schema, rx, WebSocketProtocols::GraphQLWS).connection_data(data);

    tx.send(
        serde_json::to_string(&value!({
            "type": "connection_init",
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(
        Some(value!({
            "type": "connection_ack",
        })),
        serde_json::from_str(&stream.next().await.unwrap().unwrap_text()).unwrap()
    );

    tx.send(
        serde_json::to_string(&value!({
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
            Some(value!({
                "type": "next",
                "id": "1",
                "payload": { "data": { "values": i } },
            })),
            serde_json::from_str(&stream.next().await.unwrap().unwrap_text()).unwrap()
        );
    }

    assert_eq!(
        Some(value!({
            "type": "complete",
            "id": "1",
        })),
        serde_json::from_str(&stream.next().await.unwrap().unwrap_text()).unwrap()
    );
}

#[tokio::test]
pub async fn test_subscription_ws_transport_error() {
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

    struct Query;

    #[Object]
    impl Query {
        async fn value(&self) -> i32 {
            10
        }
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        async fn events(&self) -> impl Stream<Item = Event> {
            futures_util::stream::iter((0..10).map(|n| Event { value: n }))
        }
    }

    let schema = Schema::new(Query, EmptyMutation, Subscription);
    let (mut tx, rx) = mpsc::unbounded();
    let mut stream = http::WebSocket::new(schema, rx, WebSocketProtocols::GraphQLWS);

    tx.send(
        serde_json::to_string(&value!({
            "type": "connection_init"
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(
        Some(value!({
            "type": "connection_ack",
        })),
        serde_json::from_str(&stream.next().await.unwrap().unwrap_text()).unwrap()
    );

    tx.send(
        serde_json::to_string(&value!({
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
            Some(value!({
                "type": "next",
                "id": "1",
                "payload": { "data": { "events": { "value": i } } },
            })),
            serde_json::from_str(&stream.next().await.unwrap().unwrap_text()).unwrap()
        );
    }

    assert_eq!(
        Some(value!({
            "type": "next",
            "id": "1",
            "payload": {
                "data": null,
                "errors": [{
                    "message": "TestError",
                    "locations": [{"line": 1, "column": 25}],
                    "path": ["events", "value"],
                }],
            },
        })),
        serde_json::from_str(&stream.next().await.unwrap().unwrap_text()).unwrap()
    );
}

#[tokio::test]
pub async fn test_subscription_init_error() {
    struct Query;

    #[Object]
    impl Query {
        async fn value(&self) -> i32 {
            10
        }
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        async fn events(&self) -> impl Stream<Item = i32> {
            futures_util::stream::once(async move { 10 })
        }
    }

    let schema = Schema::new(Query, EmptyMutation, Subscription);
    let (mut tx, rx) = mpsc::unbounded();
    let mut stream = http::WebSocket::new(schema, rx, WebSocketProtocols::GraphQLWS)
        .on_connection_init(|_| async move { Err("Error!".into()) });

    tx.send(
        serde_json::to_string(&value!({
            "type": "connection_init"
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(
        (1002, "Error!".to_string()),
        dbg!(stream.next().await.unwrap()).unwrap_close()
    );
}

#[tokio::test]
pub async fn test_subscription_too_many_initialisation_requests_error() {
    struct Query;

    #[Object]
    impl Query {
        async fn value(&self) -> i32 {
            10
        }
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        async fn events(&self) -> impl Stream<Item = i32> {
            futures_util::stream::once(async move { 10 })
        }
    }

    let schema = Schema::new(Query, EmptyMutation, Subscription);
    let (mut tx, rx) = mpsc::unbounded();
    let mut stream = http::WebSocket::new(schema, rx, WebSocketProtocols::GraphQLWS);

    tx.send(
        serde_json::to_string(&value!({
            "type": "connection_init"
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(
        Some(value!({
            "type": "connection_ack",
        })),
        serde_json::from_str(&stream.next().await.unwrap().unwrap_text()).unwrap()
    );

    tx.send(
        serde_json::to_string(&value!({
            "type": "connection_init"
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(
        (4429, "Too many initialisation requests.".to_string()),
        stream.next().await.unwrap().unwrap_close()
    );
}

#[tokio::test]
pub async fn test_query_over_websocket() {
    struct Query;

    #[Object]
    impl Query {
        async fn value(&self) -> i32 {
            999
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let (mut tx, rx) = mpsc::unbounded();
    let mut stream = http::WebSocket::new(schema, rx, WebSocketProtocols::GraphQLWS);

    tx.send(
        serde_json::to_string(&value!({
            "type": "connection_init",
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(
        Some(value!({
        "type": "connection_ack",
        })),
        serde_json::from_str(&stream.next().await.unwrap().unwrap_text()).unwrap()
    );

    tx.send(
        serde_json::to_string(&value!({
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
        Some(value!({
            "type": "next",
            "id": "1",
            "payload": { "data": { "value": 999 } },
        })),
        serde_json::from_str(&stream.next().await.unwrap().unwrap_text()).unwrap()
    );

    assert_eq!(
        Some(value!({
            "type": "complete",
            "id": "1",
        })),
        serde_json::from_str(&stream.next().await.unwrap().unwrap_text()).unwrap()
    );
}

#[tokio::test]
pub async fn test_start_before_connection_init() {
    struct Query;

    #[Object]
    impl Query {
        async fn value(&self) -> i32 {
            999
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let (mut tx, rx) = mpsc::unbounded();
    let mut stream = http::WebSocket::new(schema, rx, WebSocketProtocols::GraphQLWS);

    tx.send(
        serde_json::to_string(&value!({
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
        stream.next().await.unwrap().unwrap_close(),
        (1011, "The handshake is not completed.".to_string())
    );
}

#[tokio::test]
pub async fn test_stream_drop() {
    type Dropped = Arc<Mutex<bool>>;

    struct TestStream {
        inner: BoxStream<'static, i32>,
        dropped: Dropped,
    }

    impl Drop for TestStream {
        fn drop(&mut self) {
            *self.dropped.lock().unwrap() = true;
        }
    }

    impl Stream for TestStream {
        type Item = i32;
        fn poll_next(
            mut self: Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Self::Item>> {
            self.inner.as_mut().poll_next(cx)
        }
    }

    struct Query;

    #[Object]
    impl Query {
        async fn value(&self) -> i32 {
            999
        }
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        async fn values(&self, ctx: &Context<'_>) -> impl Stream<Item = i32> {
            TestStream {
                inner: Box::pin(async_stream::stream! {
                    loop {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        yield 10;
                    }
                }),
                dropped: ctx.data_unchecked::<Dropped>().clone(),
            }
        }
    }

    let dropped = Dropped::default();
    let schema = Schema::build(Query, EmptyMutation, Subscription)
        .data(dropped.clone())
        .finish();
    let (mut tx, rx) = mpsc::unbounded();
    let mut stream = http::WebSocket::new(schema, rx, WebSocketProtocols::GraphQLWS);

    tx.send(
        serde_json::to_string(&value!({
            "type": "connection_init",
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&stream.next().await.unwrap().unwrap_text())
            .unwrap(),
        serde_json::json!({
            "type": "connection_ack",
        }),
    );

    tx.send(
        serde_json::to_string(&value!({
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

    for _ in 0..5 {
        assert_eq!(
            Some(value!({
                "type": "next",
                "id": "1",
                "payload": { "data": { "values": 10 } },
            })),
            serde_json::from_str(&stream.next().await.unwrap().unwrap_text()).unwrap()
        );
    }

    tx.send(
        serde_json::to_string(&value!({
            "type": "stop",
            "id": "1",
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    loop {
        let value = serde_json::from_str(&stream.next().await.unwrap().unwrap_text()).unwrap();
        if value
            == Some(value!({
                "type": "next",
                "id": "1",
                "payload": { "data": { "values": 10 } },
            }))
        {
            continue;
        } else {
            assert_eq!(
                Some(value!({
                    "type": "complete",
                    "id": "1",
                })),
                value
            );
            break;
        }
    }

    assert!(*dropped.lock().unwrap());
}

#[tokio::test]
pub async fn test_ping_pong() {
    struct Query;

    #[Object]
    impl Query {
        async fn value(&self) -> i32 {
            10
        }
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        async fn values(&self) -> impl Stream<Item = i32> {
            futures_util::stream::iter(0..10)
        }
    }

    let schema = Schema::new(Query, EmptyMutation, Subscription);
    let (mut tx, rx) = mpsc::unbounded();
    let mut stream = http::WebSocket::new(schema, rx, WebSocketProtocols::GraphQLWS);

    tx.send(
        serde_json::to_string(&value!({
            "type": "connection_init",
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&stream.next().await.unwrap().unwrap_text())
            .unwrap(),
        serde_json::json!({
            "type": "connection_ack",
        }),
    );

    for _ in 0..5 {
        tx.send(
            serde_json::to_string(&value!({
                "type": "ping",
            }))
            .unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&stream.next().await.unwrap().unwrap_text())
                .unwrap(),
            serde_json::json!({
                "type": "pong",
            }),
        );
    }

    tx.send(
        serde_json::to_string(&value!({
            "type": "pong",
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    assert!(
        tokio::time::timeout(Duration::from_millis(100), stream.next())
            .await
            .is_err()
    );
}

#[tokio::test]
pub async fn test_connection_terminate() {
    struct Query;

    #[Object]
    impl Query {
        async fn value(&self) -> i32 {
            10
        }
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        async fn values(&self) -> impl Stream<Item = i32> {
            futures_util::stream::iter(0..10)
        }
    }

    let schema = Schema::new(Query, EmptyMutation, Subscription);
    let (mut tx, rx) = mpsc::unbounded();
    let mut stream = http::WebSocket::new(schema, rx, WebSocketProtocols::GraphQLWS);

    tx.send(
        serde_json::to_string(&value!({
            "type": "connection_init",
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&stream.next().await.unwrap().unwrap_text())
            .unwrap(),
        serde_json::json!({
            "type": "connection_ack",
        }),
    );

    tx.send(
        serde_json::to_string(&value!({
            "type": "connection_terminate",
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    assert!(stream.next().await.is_none());
    assert!(stream.next().await.is_none());
}

#[tokio::test]
pub async fn test_keepalive_timeout_1() {
    struct Query;

    #[Object]
    impl Query {
        async fn value(&self) -> i32 {
            10
        }
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        async fn values(&self) -> impl Stream<Item = i32> {
            futures_util::stream::iter(0..10)
        }
    }

    let schema = Schema::new(Query, EmptyMutation, Subscription);
    let (mut tx, rx) = mpsc::unbounded();
    let mut stream = http::WebSocket::new(schema, rx, WebSocketProtocols::GraphQLWS)
        .keepalive_timeout(Duration::from_secs(3));

    tx.send(
        serde_json::to_string(&value!({
            "type": "connection_init",
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&stream.next().await.unwrap().unwrap_text())
            .unwrap(),
        serde_json::json!({
            "type": "connection_ack",
        }),
    );

    assert!(tokio::time::timeout(Duration::from_secs(2), stream.next())
        .await
        .is_err());

    assert_eq!(
        tokio::time::timeout(Duration::from_secs(2), stream.next()).await,
        Ok(Some(WsMessage::Close(3008, "timeout".to_string())))
    );

    assert!(stream.next().await.is_none());
}

#[tokio::test]
pub async fn test_keepalive_timeout_2() {
    struct Query;

    #[Object]
    impl Query {
        async fn value(&self) -> i32 {
            10
        }
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        async fn values(&self) -> impl Stream<Item = i32> {
            futures_util::stream::iter(0..10)
        }
    }

    let schema = Schema::new(Query, EmptyMutation, Subscription);
    let (mut tx, rx) = mpsc::unbounded();
    let mut stream = http::WebSocket::new(schema, rx, WebSocketProtocols::GraphQLWS)
        .keepalive_timeout(Duration::from_secs(3));

    tx.send(
        serde_json::to_string(&value!({
            "type": "connection_init",
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&stream.next().await.unwrap().unwrap_text())
            .unwrap(),
        serde_json::json!({
            "type": "connection_ack",
        }),
    );

    tokio::time::sleep(Duration::from_secs(2)).await;

    tx.send(
        serde_json::to_string(&value!({
            "type": "ping",
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&stream.next().await.unwrap().unwrap_text())
            .unwrap(),
        serde_json::json!({
            "type": "pong",
        }),
    );

    assert!(tokio::time::timeout(Duration::from_secs(2), stream.next())
        .await
        .is_err());

    assert_eq!(
        tokio::time::timeout(Duration::from_secs(2), stream.next()).await,
        Ok(Some(WsMessage::Close(3008, "timeout".to_string())))
    );

    assert!(stream.next().await.is_none());
}

#[tokio::test]
pub async fn test_on_ping() {
    struct Query;

    #[Object]
    impl Query {
        async fn value(&self) -> i32 {
            10
        }
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        async fn values(&self) -> impl Stream<Item = i32> {
            futures_util::stream::iter(0..10)
        }
    }

    let schema = Schema::new(Query, EmptyMutation, Subscription);
    let (mut tx, rx) = mpsc::unbounded();
    let mut stream = http::WebSocket::new(schema, rx, WebSocketProtocols::GraphQLWS).on_ping(
        |_, payload| async move {
            assert_eq!(payload, Some(serde_json::json!({"a": 1, "b": "abc"})));
            Ok(Some(serde_json::json!({ "a": "abc", "b": 1 })))
        },
    );

    tx.send(
        serde_json::to_string(&value!({
            "type": "connection_init",
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&stream.next().await.unwrap().unwrap_text())
            .unwrap(),
        serde_json::json!({
            "type": "connection_ack",
        }),
    );

    tx.send(
        serde_json::to_string(&value!({
            "type": "ping",
            "payload": {
                "a": 1,
                "b": "abc"
            }
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&stream.next().await.unwrap().unwrap_text())
            .unwrap(),
        serde_json::json!({
            "type": "pong",
            "payload": {
                "a": "abc",
                "b": 1
            }
        }),
    );
}
