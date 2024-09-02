use std::{future::Future, str::FromStr, time::Duration};

use async_graphql::{
    http::{
        default_on_connection_init, default_on_ping, DefaultOnConnInitType, DefaultOnPingType,
        WebSocket as AGWebSocket, WebSocketProtocols, WsMessage, ALL_WEBSOCKET_PROTOCOLS,
    },
    Data, Executor, Result,
};
use futures_util::{future, StreamExt};
use tide::Endpoint;
use tide_websockets::{tungstenite::protocol::CloseFrame, Message};

/// A GraphQL subscription endpoint builder.
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
pub struct GraphQLSubscription<E, OnConnInit, OnPing> {
    executor: E,
    on_connection_init: OnConnInit,
    on_ping: OnPing,
    keepalive_timeout: Option<Duration>,
}

impl<E> GraphQLSubscription<E, DefaultOnConnInitType, DefaultOnPingType>
where
    E: Executor,
{
    /// Create a [`GraphQLSubscription`] object.
    pub fn new(executor: E) -> Self {
        GraphQLSubscription {
            executor,
            on_connection_init: default_on_connection_init,
            on_ping: default_on_ping,
            keepalive_timeout: None,
        }
    }
}

impl<E, OnConnInit, OnConnInitFut, OnPing, OnPingFut> GraphQLSubscription<E, OnConnInit, OnPing>
where
    E: Executor,
    OnConnInit: Fn(serde_json::Value) -> OnConnInitFut + Clone + Send + Sync + 'static,
    OnConnInitFut: Future<Output = async_graphql::Result<Data>> + Send + 'static,
    OnPing: FnOnce(Option<&Data>, Option<serde_json::Value>) -> OnPingFut
        + Clone
        + Send
        + Sync
        + 'static,
    OnPingFut: Future<Output = async_graphql::Result<Option<serde_json::Value>>> + Send + 'static,
{
    /// Specify a callback function to be called when the connection is
    /// initialized.
    ///
    /// You can get something from the payload of [`GQL_CONNECTION_INIT` message](https://github.com/apollographql/subscriptions-transport-ws/blob/master/PROTOCOL.md#gql_connection_init) to create [`Data`].
    /// The data returned by this callback function will be merged with the data
    /// specified by [`with_data`].
    #[must_use]
    pub fn on_connection_init<F, R>(self, callback: F) -> GraphQLSubscription<E, F, OnPing>
    where
        F: Fn(serde_json::Value) -> R + Clone + Send + Sync + 'static,
        R: Future<Output = async_graphql::Result<Data>> + Send + 'static,
    {
        GraphQLSubscription {
            executor: self.executor,
            on_connection_init: callback,
            on_ping: self.on_ping,
            keepalive_timeout: self.keepalive_timeout,
        }
    }

    /// Specify a ping callback function.
    ///
    /// This function if present, will be called with the data sent by the
    /// client in the [`Ping` message](https://github.com/enisdenjo/graphql-ws/blob/master/PROTOCOL.md#ping).
    ///
    /// The function should return the data to be sent in the [`Pong` message](https://github.com/enisdenjo/graphql-ws/blob/master/PROTOCOL.md#pong).
    ///
    /// NOTE: Only used for the `graphql-ws` protocol.
    #[must_use]
    pub fn on_ping<F, R>(self, callback: F) -> GraphQLSubscription<E, OnConnInit, F>
    where
        F: FnOnce(Option<&Data>, Option<serde_json::Value>) -> R + Send + Sync + Clone + 'static,
        R: Future<Output = Result<Option<serde_json::Value>>> + Send + 'static,
    {
        GraphQLSubscription {
            executor: self.executor,
            on_connection_init: self.on_connection_init,
            on_ping: callback,
            keepalive_timeout: self.keepalive_timeout,
        }
    }

    /// Sets a timeout for receiving an acknowledgement of the keep-alive ping.
    ///
    /// If the ping is not acknowledged within the timeout, the connection will
    /// be closed.
    ///
    /// NOTE: Only used for the `graphql-ws` protocol.
    #[must_use]
    pub fn keepalive_timeout(self, timeout: impl Into<Option<Duration>>) -> Self {
        Self {
            keepalive_timeout: timeout.into(),
            ..self
        }
    }

    /// Consumes this builder to create a tide endpoint.
    pub fn build<S>(self) -> impl Endpoint<S>
    where
        S: Send + Sync + Clone + 'static,
    {
        tide_websockets::WebSocket::<S, _>::new(move |request, connection| {
            let executor = self.executor.clone();
            let on_connection_init = self.on_connection_init.clone();
            let on_ping = self.on_ping.clone();
            async move {
                let protocol = match request
                    .header("sec-websocket-protocol")
                    .map(|value| value.as_str())
                    .and_then(|protocols| {
                        protocols
                            .split(',')
                            .find_map(|p| WebSocketProtocols::from_str(p.trim()).ok())
                    }) {
                    Some(protocol) => protocol,
                    None => {
                        // default to the prior standard
                        WebSocketProtocols::SubscriptionsTransportWS
                    }
                };

                let sink = connection.clone();
                let mut stream = AGWebSocket::new(
                    executor.clone(),
                    connection
                        .take_while(|msg| future::ready(msg.is_ok()))
                        .map(Result::unwrap)
                        .map(Message::into_data),
                    protocol,
                )
                .on_connection_init(on_connection_init)
                .on_ping(on_ping)
                .keepalive_timeout(self.keepalive_timeout);

                while let Some(data) = stream.next().await {
                    match data {
                        WsMessage::Text(text) => {
                            if sink.send_string(text).await.is_err() {
                                break;
                            }
                        }
                        WsMessage::Close(code, msg) => {
                            let _ = sink
                                .send(Message::Close(Some(CloseFrame {
                                    code: code.into(),
                                    reason: msg.into(),
                                })))
                                .await;
                            break;
                        }
                    }
                }

                Ok(())
            }
        })
        .with_protocols(&ALL_WEBSOCKET_PROTOCOLS)
    }
}
