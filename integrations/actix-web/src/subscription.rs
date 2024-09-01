use std::{
    future::Future,
    str::FromStr,
    time::{Duration, Instant},
};

use actix::{
    Actor, ActorContext, ActorFutureExt, ActorStreamExt, AsyncContext, ContextFutureSpawner,
    StreamHandler, WrapFuture, WrapStream,
};
use actix_http::{error::PayloadError, ws};
use actix_web::{web::Bytes, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws::{CloseReason, Message, ProtocolError, WebsocketContext};
use async_graphql::{
    http::{
        default_on_connection_init, default_on_ping, DefaultOnConnInitType, DefaultOnPingType,
        WebSocket, WebSocketProtocols, WsMessage, ALL_WEBSOCKET_PROTOCOLS,
    },
    Data, Executor, Result,
};
use futures_util::stream::Stream;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(thiserror::Error, Debug)]
#[error("failed to parse graphql protocol")]
pub struct ParseGraphQLProtocolError;

/// A builder for websocket subscription actor.
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
    OnPingFut: Future<Output = async_graphql::Result<Option<serde_json::Value>>> + Send + 'static,
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

    /// Start the subscription actor.
    pub fn start<S>(self, request: &HttpRequest, stream: S) -> Result<HttpResponse, Error>
    where
        S: Stream<Item = Result<Bytes, PayloadError>> + 'static,
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

        let actor = GraphQLSubscriptionActor {
            executor: self.executor,
            data: Some(self.data),
            protocol,
            last_heartbeat: Instant::now(),
            messages: None,
            on_connection_init: Some(self.on_connection_init),
            on_ping: self.on_ping,
            keepalive_timeout: self.keepalive_timeout,
            continuation: Vec::new(),
        };

        actix_web_actors::ws::WsResponseBuilder::new(actor, request, stream)
            .protocols(&ALL_WEBSOCKET_PROTOCOLS)
            .start()
    }
}

struct GraphQLSubscriptionActor<E, OnInit, OnPing> {
    executor: E,
    data: Option<Data>,
    protocol: WebSocketProtocols,
    last_heartbeat: Instant,
    messages: Option<async_channel::Sender<Vec<u8>>>,
    on_connection_init: Option<OnInit>,
    on_ping: OnPing,
    keepalive_timeout: Option<Duration>,
    continuation: Vec<u8>,
}

impl<E, OnInit, OnInitFut, OnPing, OnPingFut> GraphQLSubscriptionActor<E, OnInit, OnPing>
where
    E: Executor,
    OnInit: FnOnce(serde_json::Value) -> OnInitFut + Unpin + Send + 'static,
    OnInitFut: Future<Output = Result<Data>> + Send + 'static,
    OnPing: FnOnce(Option<&Data>, Option<serde_json::Value>) -> OnPingFut
        + Clone
        + Unpin
        + Send
        + 'static,
    OnPingFut: Future<Output = Result<Option<serde_json::Value>>> + Send + 'static,
{
    fn send_heartbeats(&self, ctx: &mut WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.last_heartbeat) > CLIENT_TIMEOUT {
                ctx.stop();
            }
            ctx.ping(b"");
        });
    }
}

impl<E, OnInit, OnInitFut, OnPing, OnPingFut> Actor for GraphQLSubscriptionActor<E, OnInit, OnPing>
where
    E: Executor,
    OnInit: FnOnce(serde_json::Value) -> OnInitFut + Unpin + Send + 'static,
    OnInitFut: Future<Output = Result<Data>> + Send + 'static,
    OnPing: FnOnce(Option<&Data>, Option<serde_json::Value>) -> OnPingFut
        + Clone
        + Unpin
        + Send
        + 'static,
    OnPingFut: Future<Output = Result<Option<serde_json::Value>>> + Send + 'static,
{
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.send_heartbeats(ctx);

        let (tx, rx) = async_channel::unbounded();

        WebSocket::new(self.executor.clone(), rx, self.protocol)
            .connection_data(self.data.take().unwrap())
            .on_connection_init(self.on_connection_init.take().unwrap())
            .on_ping(self.on_ping.clone())
            .keepalive_timeout(self.keepalive_timeout)
            .into_actor(self)
            .map(|response, _act, ctx| match response {
                WsMessage::Text(text) => ctx.text(text),
                WsMessage::Close(code, msg) => ctx.close(Some(CloseReason {
                    code: code.into(),
                    description: Some(msg),
                })),
            })
            .finish()
            .spawn(ctx);

        self.messages = Some(tx);
    }
}

impl<E, OnInit, OnInitFut, OnPing, OnPingFut> StreamHandler<Result<Message, ProtocolError>>
    for GraphQLSubscriptionActor<E, OnInit, OnPing>
where
    E: Executor,
    OnInit: FnOnce(serde_json::Value) -> OnInitFut + Unpin + Send + 'static,
    OnInitFut: Future<Output = Result<Data>> + Send + 'static,
    OnPing: FnOnce(Option<&Data>, Option<serde_json::Value>) -> OnPingFut
        + Clone
        + Unpin
        + Send
        + 'static,
    OnPingFut: Future<Output = async_graphql::Result<Option<serde_json::Value>>> + Send + 'static,
{
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        let message = match msg {
            Message::Ping(msg) => {
                self.last_heartbeat = Instant::now();
                ctx.pong(&msg);
                None
            }
            Message::Pong(_) => {
                self.last_heartbeat = Instant::now();
                None
            }
            Message::Continuation(item) => match item {
                ws::Item::FirstText(bytes) | ws::Item::FirstBinary(bytes) => {
                    self.continuation = bytes.to_vec();
                    None
                }
                ws::Item::Continue(bytes) => {
                    self.continuation.extend_from_slice(&bytes);
                    None
                }
                ws::Item::Last(bytes) => {
                    self.continuation.extend_from_slice(&bytes);
                    Some(std::mem::take(&mut self.continuation))
                }
            },
            Message::Text(s) => Some(s.into_bytes().to_vec()),
            Message::Binary(bytes) => Some(bytes.to_vec()),
            Message::Close(_) => {
                ctx.stop();
                None
            }
            Message::Nop => None,
        };

        if let Some(message) = message {
            let sender = self.messages.as_ref().unwrap().clone();

            async move { sender.send(message).await }
                .into_actor(self)
                .map(|res, _actor, ctx| match res {
                    Ok(()) => {}
                    Err(_) => ctx.stop(),
                })
                .spawn(ctx)
        }
    }
}
