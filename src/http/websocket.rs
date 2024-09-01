//! WebSocket transport for subscription

use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use futures_timer::Delay;
use futures_util::{
    future::{BoxFuture, Ready},
    stream::Stream,
    FutureExt, StreamExt,
};
use pin_project_lite::pin_project;
use serde::{Deserialize, Serialize};

use crate::{Data, Error, Executor, Request, Response, Result};

/// All known protocols based on WebSocket.
pub const ALL_WEBSOCKET_PROTOCOLS: [&str; 2] = ["graphql-transport-ws", "graphql-ws"];

/// An enum representing the various forms of a WebSocket message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WsMessage {
    /// A text WebSocket message
    Text(String),

    /// A close message with the close frame.
    Close(u16, String),
}

impl WsMessage {
    /// Returns the contained [WsMessage::Text] value, consuming the `self`
    /// value.
    ///
    /// Because this function may panic, its use is generally discouraged.
    ///
    /// # Panics
    ///
    /// Panics if the self value not equals [WsMessage::Text].
    pub fn unwrap_text(self) -> String {
        match self {
            Self::Text(text) => text,
            Self::Close(_, _) => panic!("Not a text message"),
        }
    }

    /// Returns the contained [WsMessage::Close] value, consuming the `self`
    /// value.
    ///
    /// Because this function may panic, its use is generally discouraged.
    ///
    /// # Panics
    ///
    /// Panics if the self value not equals [WsMessage::Close].
    pub fn unwrap_close(self) -> (u16, String) {
        match self {
            Self::Close(code, msg) => (code, msg),
            Self::Text(_) => panic!("Not a close message"),
        }
    }
}

struct Timer {
    interval: Duration,
    delay: Delay,
}

impl Timer {
    #[inline]
    fn new(interval: Duration) -> Self {
        Self {
            interval,
            delay: Delay::new(interval),
        }
    }

    #[inline]
    fn reset(&mut self) {
        self.delay.reset(self.interval);
    }
}

impl Stream for Timer {
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = &mut *self;
        match this.delay.poll_unpin(cx) {
            Poll::Ready(_) => {
                this.delay.reset(this.interval);
                Poll::Ready(Some(()))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

pin_project! {
    /// A GraphQL connection over websocket.
    ///
    /// # References
    ///
    /// - [subscriptions-transport-ws](https://github.com/apollographql/subscriptions-transport-ws/blob/master/PROTOCOL.md)
    /// - [graphql-ws](https://github.com/enisdenjo/graphql-ws/blob/master/PROTOCOL.md)
    pub struct WebSocket<S, E, OnInit, OnPing> {
        on_connection_init: Option<OnInit>,
        on_ping: OnPing,
        init_fut: Option<BoxFuture<'static, Result<Data>>>,
        ping_fut: Option<BoxFuture<'static, Result<Option<serde_json::Value>>>>,
        connection_data: Option<Data>,
        data: Option<Arc<Data>>,
        executor: E,
        streams: HashMap<String, Pin<Box<dyn Stream<Item = Response> + Send>>>,
        #[pin]
        stream: S,
        protocol: Protocols,
        last_msg_at: Instant,
        keepalive_timer: Option<Timer>,
        close: bool,
    }
}

type MessageMapStream<S> =
    futures_util::stream::Map<S, fn(<S as Stream>::Item) -> serde_json::Result<ClientMessage>>;

/// Default connection initializer type.
pub type DefaultOnConnInitType = fn(serde_json::Value) -> Ready<Result<Data>>;

/// Default ping handler type.
pub type DefaultOnPingType =
    fn(Option<&Data>, Option<serde_json::Value>) -> Ready<Result<Option<serde_json::Value>>>;

/// Default connection initializer function.
pub fn default_on_connection_init(_: serde_json::Value) -> Ready<Result<Data>> {
    futures_util::future::ready(Ok(Data::default()))
}

/// Default ping handler function.
pub fn default_on_ping(
    _: Option<&Data>,
    _: Option<serde_json::Value>,
) -> Ready<Result<Option<serde_json::Value>>> {
    futures_util::future::ready(Ok(None))
}

impl<S, E> WebSocket<S, E, DefaultOnConnInitType, DefaultOnPingType>
where
    E: Executor,
    S: Stream<Item = serde_json::Result<ClientMessage>>,
{
    /// Create a new websocket from [`ClientMessage`] stream.
    pub fn from_message_stream(executor: E, stream: S, protocol: Protocols) -> Self {
        WebSocket {
            on_connection_init: Some(default_on_connection_init),
            on_ping: default_on_ping,
            init_fut: None,
            ping_fut: None,
            connection_data: None,
            data: None,
            executor,
            streams: HashMap::new(),
            stream,
            protocol,
            last_msg_at: Instant::now(),
            keepalive_timer: None,
            close: false,
        }
    }
}

impl<S, E> WebSocket<MessageMapStream<S>, E, DefaultOnConnInitType, DefaultOnPingType>
where
    E: Executor,
    S: Stream,
    S::Item: AsRef<[u8]>,
{
    /// Create a new websocket from bytes stream.
    pub fn new(executor: E, stream: S, protocol: Protocols) -> Self {
        let stream = stream
            .map(ClientMessage::from_bytes as fn(S::Item) -> serde_json::Result<ClientMessage>);
        WebSocket::from_message_stream(executor, stream, protocol)
    }
}

impl<S, E, OnInit, OnPing> WebSocket<S, E, OnInit, OnPing>
where
    E: Executor,
    S: Stream<Item = serde_json::Result<ClientMessage>>,
{
    /// Specify a connection data.
    ///
    /// This data usually comes from HTTP requests.
    /// When the `GQL_CONNECTION_INIT` message is received, this data will be
    /// merged with the data returned by the closure specified by
    /// `with_initializer` into the final subscription context data.
    #[must_use]
    pub fn connection_data(mut self, data: Data) -> Self {
        self.connection_data = Some(data);
        self
    }

    /// Specify a connection initialize callback function.
    ///
    /// This function if present, will be called with the data sent by the
    /// client in the [`GQL_CONNECTION_INIT` message](https://github.com/apollographql/subscriptions-transport-ws/blob/master/PROTOCOL.md#gql_connection_init).
    /// From that point on the returned data will be accessible to all requests.
    #[must_use]
    pub fn on_connection_init<F, R>(self, callback: F) -> WebSocket<S, E, F, OnPing>
    where
        F: FnOnce(serde_json::Value) -> R + Send + 'static,
        R: Future<Output = Result<Data>> + Send + 'static,
    {
        WebSocket {
            on_connection_init: Some(callback),
            on_ping: self.on_ping,
            init_fut: self.init_fut,
            ping_fut: self.ping_fut,
            connection_data: self.connection_data,
            data: self.data,
            executor: self.executor,
            streams: self.streams,
            stream: self.stream,
            protocol: self.protocol,
            last_msg_at: self.last_msg_at,
            keepalive_timer: self.keepalive_timer,
            close: self.close,
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
    pub fn on_ping<F, R>(self, callback: F) -> WebSocket<S, E, OnInit, F>
    where
        F: FnOnce(Option<&Data>, Option<serde_json::Value>) -> R + Send + Clone + 'static,
        R: Future<Output = Result<Option<serde_json::Value>>> + Send + 'static,
    {
        WebSocket {
            on_connection_init: self.on_connection_init,
            on_ping: callback,
            init_fut: self.init_fut,
            ping_fut: self.ping_fut,
            connection_data: self.connection_data,
            data: self.data,
            executor: self.executor,
            streams: self.streams,
            stream: self.stream,
            protocol: self.protocol,
            last_msg_at: self.last_msg_at,
            keepalive_timer: self.keepalive_timer,
            close: self.close,
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
            keepalive_timer: timeout.into().map(Timer::new),
            ..self
        }
    }
}

impl<S, E, OnInit, InitFut, OnPing, PingFut> Stream for WebSocket<S, E, OnInit, OnPing>
where
    E: Executor,
    S: Stream<Item = serde_json::Result<ClientMessage>>,
    OnInit: FnOnce(serde_json::Value) -> InitFut + Send + 'static,
    InitFut: Future<Output = Result<Data>> + Send + 'static,
    OnPing: FnOnce(Option<&Data>, Option<serde_json::Value>) -> PingFut + Clone + Send + 'static,
    PingFut: Future<Output = Result<Option<serde_json::Value>>> + Send + 'static,
{
    type Item = WsMessage;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        if *this.close {
            return Poll::Ready(None);
        }

        if let Some(keepalive_timer) = this.keepalive_timer {
            if let Poll::Ready(Some(())) = keepalive_timer.poll_next_unpin(cx) {
                return match this.protocol {
                    Protocols::SubscriptionsTransportWS => {
                        *this.close = true;
                        Poll::Ready(Some(WsMessage::Text(
                            serde_json::to_string(&ServerMessage::ConnectionError {
                                payload: Error::new("timeout"),
                            })
                            .unwrap(),
                        )))
                    }
                    Protocols::GraphQLWS => {
                        *this.close = true;
                        Poll::Ready(Some(WsMessage::Close(3008, "timeout".to_string())))
                    }
                };
            }
        }

        if this.init_fut.is_none() && this.ping_fut.is_none() {
            while let Poll::Ready(message) = Pin::new(&mut this.stream).poll_next(cx) {
                let message = match message {
                    Some(message) => message,
                    None => return Poll::Ready(None),
                };

                let message: ClientMessage = match message {
                    Ok(message) => message,
                    Err(err) => {
                        *this.close = true;
                        return Poll::Ready(Some(WsMessage::Close(1002, err.to_string())));
                    }
                };

                *this.last_msg_at = Instant::now();
                if let Some(keepalive_timer) = this.keepalive_timer {
                    keepalive_timer.reset();
                }

                match message {
                    ClientMessage::ConnectionInit { payload } => {
                        if let Some(on_connection_init) = this.on_connection_init.take() {
                            *this.init_fut = Some(Box::pin(async move {
                                on_connection_init(payload.unwrap_or_default()).await
                            }));
                            break;
                        } else {
                            *this.close = true;
                            match this.protocol {
                                Protocols::SubscriptionsTransportWS => {
                                    return Poll::Ready(Some(WsMessage::Text(
                                        serde_json::to_string(&ServerMessage::ConnectionError {
                                            payload: Error::new(
                                                "Too many initialisation requests.",
                                            ),
                                        })
                                        .unwrap(),
                                    )));
                                }
                                Protocols::GraphQLWS => {
                                    return Poll::Ready(Some(WsMessage::Close(
                                        4429,
                                        "Too many initialisation requests.".to_string(),
                                    )));
                                }
                            }
                        }
                    }
                    ClientMessage::Start {
                        id,
                        payload: request,
                    } => {
                        if let Some(data) = this.data.clone() {
                            this.streams.insert(
                                id,
                                Box::pin(this.executor.execute_stream(request, Some(data))),
                            );
                        } else {
                            *this.close = true;
                            return Poll::Ready(Some(WsMessage::Close(
                                1011,
                                "The handshake is not completed.".to_string(),
                            )));
                        }
                    }
                    ClientMessage::Stop { id } => {
                        if this.streams.remove(&id).is_some() {
                            return Poll::Ready(Some(WsMessage::Text(
                                serde_json::to_string(&ServerMessage::Complete { id: &id })
                                    .unwrap(),
                            )));
                        }
                    }
                    // Note: in the revised `graphql-ws` spec, there is no equivalent to the
                    // `CONNECTION_TERMINATE` `client -> server` message; rather, disconnection is
                    // handled by disconnecting the websocket
                    ClientMessage::ConnectionTerminate => {
                        *this.close = true;
                        return Poll::Ready(None);
                    }
                    // Pong must be sent in response from the receiving party as soon as possible.
                    ClientMessage::Ping { payload } => {
                        let on_ping = this.on_ping.clone();
                        let data = this.data.clone();
                        *this.ping_fut =
                            Some(Box::pin(
                                async move { on_ping(data.as_deref(), payload).await },
                            ));
                        break;
                    }
                    ClientMessage::Pong { .. } => {
                        // Do nothing...
                    }
                }
            }
        }

        if let Some(init_fut) = this.init_fut {
            return init_fut.poll_unpin(cx).map(|res| {
                *this.init_fut = None;
                match res {
                    Ok(data) => {
                        let mut ctx_data = this.connection_data.take().unwrap_or_default();
                        ctx_data.merge(data);
                        *this.data = Some(Arc::new(ctx_data));
                        Some(WsMessage::Text(
                            serde_json::to_string(&ServerMessage::ConnectionAck).unwrap(),
                        ))
                    }
                    Err(err) => {
                        *this.close = true;
                        match this.protocol {
                            Protocols::SubscriptionsTransportWS => Some(WsMessage::Text(
                                serde_json::to_string(&ServerMessage::ConnectionError {
                                    payload: Error::new(err.message),
                                })
                                .unwrap(),
                            )),
                            Protocols::GraphQLWS => Some(WsMessage::Close(1002, err.message)),
                        }
                    }
                }
            });
        }

        if let Some(ping_fut) = this.ping_fut {
            return ping_fut.poll_unpin(cx).map(|res| {
                *this.ping_fut = None;
                match res {
                    Ok(payload) => Some(WsMessage::Text(
                        serde_json::to_string(&ServerMessage::Pong { payload }).unwrap(),
                    )),
                    Err(err) => {
                        *this.close = true;
                        match this.protocol {
                            Protocols::SubscriptionsTransportWS => Some(WsMessage::Text(
                                serde_json::to_string(&ServerMessage::ConnectionError {
                                    payload: Error::new(err.message),
                                })
                                .unwrap(),
                            )),
                            Protocols::GraphQLWS => Some(WsMessage::Close(1002, err.message)),
                        }
                    }
                }
            });
        }

        for (id, stream) in &mut *this.streams {
            match Pin::new(stream).poll_next(cx) {
                Poll::Ready(Some(payload)) => {
                    return Poll::Ready(Some(WsMessage::Text(
                        serde_json::to_string(&this.protocol.next_message(id, payload)).unwrap(),
                    )));
                }
                Poll::Ready(None) => {
                    let id = id.clone();
                    this.streams.remove(&id);
                    return Poll::Ready(Some(WsMessage::Text(
                        serde_json::to_string(&ServerMessage::Complete { id: &id }).unwrap(),
                    )));
                }
                Poll::Pending => {}
            }
        }

        Poll::Pending
    }
}

/// Specification of which GraphQL Over WebSockets protocol is being utilized
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Protocols {
    /// [subscriptions-transport-ws protocol](https://github.com/apollographql/subscriptions-transport-ws/blob/master/PROTOCOL.md).
    SubscriptionsTransportWS,
    /// [graphql-ws protocol](https://github.com/enisdenjo/graphql-ws/blob/master/PROTOCOL.md).
    GraphQLWS,
}

impl Protocols {
    /// Returns the `Sec-WebSocket-Protocol` header value for the protocol
    pub fn sec_websocket_protocol(&self) -> &'static str {
        match self {
            Protocols::SubscriptionsTransportWS => "graphql-ws",
            Protocols::GraphQLWS => "graphql-transport-ws",
        }
    }

    #[inline]
    fn next_message<'s>(&self, id: &'s str, payload: Response) -> ServerMessage<'s> {
        match self {
            Protocols::SubscriptionsTransportWS => ServerMessage::Data { id, payload },
            Protocols::GraphQLWS => ServerMessage::Next { id, payload },
        }
    }
}

impl std::str::FromStr for Protocols {
    type Err = Error;

    fn from_str(protocol: &str) -> Result<Self, Self::Err> {
        if protocol.eq_ignore_ascii_case("graphql-ws") {
            Ok(Protocols::SubscriptionsTransportWS)
        } else if protocol.eq_ignore_ascii_case("graphql-transport-ws") {
            Ok(Protocols::GraphQLWS)
        } else {
            Err(Error::new(format!(
                "Unsupported Sec-WebSocket-Protocol: {}",
                protocol
            )))
        }
    }
}

/// A websocket message received from the client
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)] // Request is at fault
pub enum ClientMessage {
    /// A new connection
    ConnectionInit {
        /// Optional init payload from the client
        payload: Option<serde_json::Value>,
    },
    /// The start of a Websocket subscription
    #[serde(alias = "subscribe")]
    Start {
        /// Message ID
        id: String,
        /// The GraphQL Request - this can be modified by protocol implementors
        /// to add files uploads.
        payload: Request,
    },
    /// The end of a Websocket subscription
    #[serde(alias = "complete")]
    Stop {
        /// Message ID
        id: String,
    },
    /// Connection terminated by the client
    ConnectionTerminate,
    /// Useful for detecting failed connections, displaying latency metrics or
    /// other types of network probing.
    ///
    /// Reference: <https://github.com/enisdenjo/graphql-ws/blob/master/PROTOCOL.md#ping>
    Ping {
        /// Additional details about the ping.
        payload: Option<serde_json::Value>,
    },
    /// The response to the Ping message.
    ///
    /// Reference: <https://github.com/enisdenjo/graphql-ws/blob/master/PROTOCOL.md#pong>
    Pong {
        /// Additional details about the pong.
        payload: Option<serde_json::Value>,
    },
}

impl ClientMessage {
    /// Creates a ClientMessage from an array of bytes
    pub fn from_bytes<T>(message: T) -> serde_json::Result<Self>
    where
        T: AsRef<[u8]>,
    {
        serde_json::from_slice(message.as_ref())
    }
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ServerMessage<'a> {
    ConnectionError {
        payload: Error,
    },
    ConnectionAck,
    /// subscriptions-transport-ws protocol next payload
    Data {
        id: &'a str,
        payload: Response,
    },
    /// graphql-ws protocol next payload
    Next {
        id: &'a str,
        payload: Response,
    },
    // Not used by this library, as it's not necessary to send
    // Error {
    //     id: &'a str,
    //     payload: serde_json::Value,
    // },
    Complete {
        id: &'a str,
    },
    /// The response to the Ping message.
    ///
    /// https://github.com/enisdenjo/graphql-ws/blob/master/PROTOCOL.md#pong
    Pong {
        #[serde(skip_serializing_if = "Option::is_none")]
        payload: Option<serde_json::Value>,
    },
    // Not used by this library
    // #[serde(rename = "ka")]
    // KeepAlive
}
