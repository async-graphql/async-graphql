use std::{
    future::Future,
    str::FromStr,
    time::{Duration, Instant},
};

use actix_web::{Error, HttpRequest, HttpResponse, web};
use async_graphql::{
    Data, Executor, Result,
    http::{
        DefaultOnConnInitType, DefaultOnPingType, WebSocket, WebSocketProtocols, WsMessage,
        default_on_connection_init, default_on_ping,
    },
    runtime::TokioTimer,
};
use futures_util::StreamExt;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(thiserror::Error, Debug)]
#[error("failed to parse graphql protocol")]
pub struct ParseGraphQLProtocolError;

/// A builder for websocket subscription handler.
pub struct GraphQLSubscription<E, OnInit, OnPing> {
    executor: E,
    data: Data,
    on_connection_init: OnInit,
    on_ping: OnPing,
    keepalive_timeout: Option<Duration>,
}

impl<E> GraphQLSubscription<E, DefaultOnConnInitType, DefaultOnPingType> {
    /// Create a GraphQL subscription builder.
    pub fn new(executor: E) -> Self {
        Self {
            executor,
            data: Default::default(),
            on_connection_init: default_on_connection_init,
            on_ping: default_on_ping,
            keepalive_timeout: None,
        }
    }
}

impl<E, OnInit, OnInitFut, OnPing, OnPingFut> GraphQLSubscription<E, OnInit, OnPing>
where
    E: Executor,
    OnInit: FnOnce(serde_json::Value) -> OnInitFut + Unpin + Send + 'static,
    OnInitFut: Future<Output = async_graphql::Result<Data>> + Send + 'static,
    OnPing: FnOnce(Option<&Data>, Option<serde_json::Value>) -> OnPingFut
        + Clone
        + Unpin
        + Send
        + 'static,
    OnPingFut: Future<Output = Result<Option<serde_json::Value>>> + Send + 'static,
{
    /// Specify the initial subscription context data, usually you can get
    /// something from the incoming request to create it.
    #[must_use]
    pub fn with_data(self, data: Data) -> Self {
        Self { data, ..self }
    }

    /// Specify a callback function to be called when the connection is
    /// initialized.
    ///
    /// You can get something from the payload of [`GQL_CONNECTION_INIT` message](https://github.com/apollographql/subscriptions-transport-ws/blob/master/PROTOCOL.md#gql_connection_init) to create [`Data`].
    /// The data returned by this callback function will be merged with the data
    /// specified by [`with_data`].
    #[must_use]
    pub fn on_connection_init<F, R>(self, callback: F) -> GraphQLSubscription<E, F, OnPing>
    where
        F: FnOnce(serde_json::Value) -> R + Unpin + Send + 'static,
        R: Future<Output = async_graphql::Result<Data>> + Send + 'static,
    {
        GraphQLSubscription {
            executor: self.executor,
            data: self.data,
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
    pub fn on_ping<F, R>(self, callback: F) -> GraphQLSubscription<E, OnInit, F>
    where
        F: FnOnce(Option<&Data>, Option<serde_json::Value>) -> R + Send + Clone + 'static,
        R: Future<Output = Result<Option<serde_json::Value>>> + Send + 'static,
    {
        GraphQLSubscription {
            executor: self.executor,
            data: self.data,
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

    /// Start the WebSocket subscription handler.
    pub fn start(self, request: &HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error>
    {
        let protocol = request
            .headers()
            .get("sec-websocket-protocol")
            .and_then(|value| value.to_str().ok())
            .and_then(|protocols| {
                protocols
                    .split(',')
                    .find_map(|p| WebSocketProtocols::from_str(p.trim()).ok())
            })
            .ok_or_else(|| actix_web::error::ErrorBadRequest(ParseGraphQLProtocolError))?;

        let (mut response, mut session, msg_stream) = actix_ws::handle(request, stream)?;

        // Echo back the negotiated subprotocol so clients can verify it.
        response.headers_mut().insert(
            actix_http::header::HeaderName::from_static("sec-websocket-protocol"),
            actix_http::header::HeaderValue::from_static(protocol.sec_websocket_protocol()),
        );

        let (tx, rx) = async_channel::unbounded::<Vec<u8>>();
        let gql_stream = WebSocket::new(self.executor, rx, protocol)
            .connection_data(self.data)
            .on_connection_init(self.on_connection_init)
            .on_ping(self.on_ping)
            .keepalive_timeout(TokioTimer::default(), self.keepalive_timeout);

        actix_web::rt::spawn(async move {
            let mut last_heartbeat = Instant::now();
            let mut continuation: Vec<u8> = Vec::new();
            let mut heartbeat = tokio::time::interval(HEARTBEAT_INTERVAL);

            let mut msg_stream = Box::pin(msg_stream);
            let mut gql_stream = Box::pin(gql_stream);

            loop {
                tokio::select! {
                    ws_msg = msg_stream.next() => {
                        match ws_msg {
                            Some(Ok(actix_ws::Message::Ping(bytes))) => {
                                last_heartbeat = Instant::now();
                                if session.pong(&bytes).await.is_err() {
                                    break;
                                }
                            }
                            Some(Ok(actix_ws::Message::Pong(_))) => {
                                last_heartbeat = Instant::now();
                            }
                            Some(Ok(actix_ws::Message::Text(text))) => {
                                if tx.send(text.into_bytes().to_vec()).await.is_err() {
                                    break;
                                }
                            }
                            Some(Ok(actix_ws::Message::Binary(bytes))) => {
                                if tx.send(bytes.to_vec()).await.is_err() {
                                    break;
                                }
                            }
                            Some(Ok(actix_ws::Message::Continuation(item))) => {
                                match item {
                                    actix_ws::Item::FirstText(bytes)
                                    | actix_ws::Item::FirstBinary(bytes) => {
                                        continuation = bytes.to_vec();
                                    }
                                    actix_ws::Item::Continue(bytes) => {
                                        continuation.extend_from_slice(&bytes);
                                    }
                                    actix_ws::Item::Last(bytes) => {
                                        continuation.extend_from_slice(&bytes);
                                        let message = std::mem::take(&mut continuation);
                                        if tx.send(message).await.is_err() {
                                            break;
                                        }
                                    }
                                }
                            }
                            Some(Ok(actix_ws::Message::Close(_))) | None => break,
                            Some(Err(_)) => break,
                            _ => {}
                        }
                    }
                    gql_response = gql_stream.next() => {
                        match gql_response {
                            Some(WsMessage::Text(text)) => {
                                if session.text(text).await.is_err() {
                                    break;
                                }
                            }
                            Some(WsMessage::Close(code, msg)) => {
                                let _ = session
                                    .close(Some(actix_ws::CloseReason {
                                        code: actix_ws::CloseCode::from(code),
                                        description: Some(msg),
                                    }))
                                    .await;
                                break;
                            }
                            None => break,
                        }
                    }
                    _ = heartbeat.tick() => {
                        if Instant::now().duration_since(last_heartbeat) > CLIENT_TIMEOUT {
                            break;
                        }
                        if session.ping(b"").await.is_err() {
                            break;
                        }
                    }
                }
            }
        });

        Ok(response)
    }
}
