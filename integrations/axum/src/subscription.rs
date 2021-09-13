use std::borrow::Cow;
use std::future::Future;

use async_graphql::http::{WebSocketProtocols, WsMessage};
use async_graphql::{Data, ObjectType, Result, Schema, SubscriptionType};
use axum::extract::ws::{CloseFrame, Message, WebSocket};
use futures_util::{future, SinkExt, StreamExt};
use headers::{Header, HeaderName, HeaderValue};

/// The Sec-Websocket-Protocol header.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct SecWebsocketProtocol(pub WebSocketProtocols);

impl Header for SecWebsocketProtocol {
    fn name() -> &'static HeaderName {
        &http::header::SEC_WEBSOCKET_PROTOCOL
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        match values.next() {
            Some(value) => {
                let value = value.to_str().map_err(|_| headers::Error::invalid())?;
                Ok(SecWebsocketProtocol(
                    value
                        .parse()
                        .ok()
                        .unwrap_or(WebSocketProtocols::SubscriptionsTransportWS),
                ))
            }
            None => Err(headers::Error::invalid()),
        }
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        values.extend(std::iter::once(HeaderValue::from_static(
            self.0.sec_websocket_protocol(),
        )))
    }
}

/// GraphQL subscription handler
pub async fn graphql_subscription<Query, Mutation, Subscription>(
    websocket: WebSocket,
    schema: Schema<Query, Mutation, Subscription>,
    protocol: SecWebsocketProtocol,
) where
    Query: ObjectType + Sync + Send + 'static,
    Mutation: ObjectType + Sync + Send + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    graphql_subscription_with_data(websocket, schema, protocol, |_| async {
        Ok(Default::default())
    })
    .await
}

/// GraphQL subscription handler
///
/// Specifies that a function converts the init payload to data.
pub async fn graphql_subscription_with_data<Query, Mutation, Subscription, F, R>(
    websocket: WebSocket,
    schema: Schema<Query, Mutation, Subscription>,
    protocol: SecWebsocketProtocol,
    initializer: F,
) where
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
    F: FnOnce(serde_json::Value) -> R + Send + 'static,
    R: Future<Output = Result<Data>> + Send + 'static,
{
    let (mut sink, stream) = websocket.split();
    let input = stream
        .take_while(|res| future::ready(res.is_ok()))
        .map(Result::unwrap)
        .filter_map(|msg| {
            if let Message::Text(_) | Message::Binary(_) = msg {
                future::ready(Some(msg))
            } else {
                future::ready(None)
            }
        })
        .map(Message::into_data);

    let mut stream =
        async_graphql::http::WebSocket::with_data(schema, input, initializer, protocol.0).map(
            |msg| match msg {
                WsMessage::Text(text) => Message::Text(text),
                WsMessage::Close(code, status) => Message::Close(Some(CloseFrame {
                    code,
                    reason: Cow::from(status),
                })),
            },
        );

    while let Some(item) = stream.next().await {
        let _ = sink.send(item).await;
    }
}
