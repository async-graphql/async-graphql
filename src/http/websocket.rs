//! WebSocket transport for subscription

use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures_util::{
    future::{BoxFuture, Ready},
    stream::Stream,
    FutureExt, StreamExt,
};
use pin_project_lite::pin_project;
use serde::{Deserialize, Serialize};

use crate::{
    Data, Error, Executor, PerMessagePostHook, PerMessagePreHook, Request, Response, Result,
};

/// All known protocols based on WebSocket.
pub const ALL_WEBSOCKET_PROTOCOLS: [&str; 2] = ["graphql-transport-ws", "graphql-ws"];

/// An enum representing the various forms of a WebSocket message.
#[derive(Clone, Debug)]
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

pin_project! {
    /// A GraphQL connection over websocket.
    ///
    /// # References
    ///
    /// - [subscriptions-transport-ws](https://github.com/apollographql/subscriptions-transport-ws/blob/master/PROTOCOL.md)
    /// - [graphql-ws](https://github.com/enisdenjo/graphql-ws/blob/master/PROTOCOL.md)
    pub struct WebSocket<S, E, OnInit> {
        on_connection_init: Option<OnInit>,
        init_fut: Option<BoxFuture<'static, Result<Data>>>,
        connection_data: Option<Data>,
        data: Option<Arc<Data>>,
        executor: E,
        streams: HashMap<String, Pin<Box<dyn Stream<Item = Response> + Send>>>,
        #[pin]
        stream: S,
        protocol: Protocols,
        per_message_pre_hook: Option<Arc<PerMessagePreHook>>,
        per_message_post_hook: Option<Arc<PerMessagePostHook>>,
    }
}

type MessageMapStream<S> =
    futures_util::stream::Map<S, fn(<S as Stream>::Item) -> serde_json::Result<ClientMessage>>;

type DefaultOnConnInitType = fn(serde_json::Value) -> Ready<Result<Data>>;

fn default_on_connection_init(_: serde_json::Value) -> Ready<Result<Data>> {
    futures_util::future::ready(Ok(Data::default()))
}

impl<S, E> WebSocket<S, E, DefaultOnConnInitType>
where
    E: Executor,
    S: Stream<Item = serde_json::Result<ClientMessage>>,
{
    /// Create a new websocket from [`ClientMessage`] stream.
    pub fn from_message_stream(executor: E, stream: S, protocol: Protocols) -> Self {
        WebSocket {
            on_connection_init: Some(default_on_connection_init),
            init_fut: None,
            connection_data: None,
            data: None,
            executor,
            streams: HashMap::new(),
            stream,
            protocol,
            per_message_pre_hook: None,
            per_message_post_hook: None,
        }
    }
}

impl<S, E> WebSocket<MessageMapStream<S>, E, DefaultOnConnInitType>
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

impl<S, E, OnInit> WebSocket<S, E, OnInit>
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
    pub fn on_connection_init<F, R>(self, callback: F) -> WebSocket<S, E, F>
    where
        F: FnOnce(serde_json::Value) -> R + Send + 'static,
        R: Future<Output = Result<Data>> + Send + 'static,
    {
        WebSocket {
            on_connection_init: Some(callback),
            init_fut: self.init_fut,
            connection_data: self.connection_data,
            data: self.data,
            executor: self.executor,
            streams: self.streams,
            stream: self.stream,
            protocol: self.protocol,
            per_message_pre_hook: self.per_message_pre_hook,
            per_message_post_hook: self.per_message_post_hook,
        }
    }

    /// Specify a per-message pre-hook.
    ///
    /// This hook will run for each message that the subscription stream emits, before running
    /// the resolvers. It can be used for starting a transaction, that all resolvers will use.
    #[must_use]
    pub fn per_message_pre_hook(
        self,
        per_message_pre_hook: Option<Arc<PerMessagePreHook>>,
    ) -> Self {
        Self {
            on_connection_init: self.on_connection_init,
            init_fut: self.init_fut,
            connection_data: self.connection_data,
            data: self.data,
            executor: self.executor,
            streams: self.streams,
            stream: self.stream,
            protocol: self.protocol,
            per_message_pre_hook,
            per_message_post_hook: self.per_message_post_hook,
        }
    }

    /// Specify a per-message post-hook.
    ///
    /// This hook will run for each message that the subscription stream emits, after running
    /// the resolvers. It can be used for committing a transaction, that all resolvers used.
    #[must_use]
    pub fn per_message_post_hook(
        self,
        per_message_post_hook: Option<Arc<PerMessagePostHook>>,
    ) -> Self {
        Self {
            on_connection_init: self.on_connection_init,
            init_fut: self.init_fut,
            connection_data: self.connection_data,
            data: self.data,
            executor: self.executor,
            streams: self.streams,
            stream: self.stream,
            protocol: self.protocol,
            per_message_pre_hook: self.per_message_pre_hook,
            per_message_post_hook,
        }
    }
}

impl<S, E, OnInit, InitFut> Stream for WebSocket<S, E, OnInit>
where
    E: Executor,
    S: Stream<Item = serde_json::Result<ClientMessage>>,
    OnInit: FnOnce(serde_json::Value) -> InitFut + Send + 'static,
    InitFut: Future<Output = Result<Data>> + Send + 'static,
{
    type Item = WsMessage;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        if this.init_fut.is_none() {
            while let Poll::Ready(message) = Pin::new(&mut this.stream).poll_next(cx) {
                let message = match message {
                    Some(message) => message,
                    None => return Poll::Ready(None),
                };

                let message: ClientMessage = match message {
                    Ok(message) => message,
                    Err(err) => return Poll::Ready(Some(WsMessage::Close(1002, err.to_string()))),
                };

                match message {
                    ClientMessage::ConnectionInit { payload } => {
                        if let Some(on_connection_init) = this.on_connection_init.take() {
                            *this.init_fut = Some(Box::pin(async move {
                                on_connection_init(payload.unwrap_or_default()).await
                            }));
                            break;
                        } else {
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
                                Box::pin(this.executor.execute_stream(
                                    request,
                                    Some(data),
                                    this.per_message_pre_hook.clone(),
                                    this.per_message_post_hook.clone(),
                                )),
                            );
                        } else {
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
                    ClientMessage::ConnectionTerminate => return Poll::Ready(None),
                    // Pong must be sent in response from the receiving party as soon as possible.
                    ClientMessage::Ping { .. } => {
                        return Poll::Ready(Some(WsMessage::Text(
                            serde_json::to_string(&ServerMessage::Pong { payload: None }).unwrap(),
                        )));
                    }
                    ClientMessage::Pong { .. } => {
                        // Do nothing...
                    }
                }
            }
        }

        if let Some(init_fut) = this.init_fut {
            if let Poll::Ready(res) = init_fut.poll_unpin(cx) {
                *this.init_fut = None;
                return match res {
                    Ok(data) => {
                        let mut ctx_data = this.connection_data.take().unwrap_or_default();
                        ctx_data.merge(data);
                        *this.data = Some(Arc::new(ctx_data));
                        Poll::Ready(Some(WsMessage::Text(
                            serde_json::to_string(&ServerMessage::ConnectionAck).unwrap(),
                        )))
                    }
                    Err(err) => match this.protocol {
                        Protocols::SubscriptionsTransportWS => Poll::Ready(Some(WsMessage::Text(
                            serde_json::to_string(&ServerMessage::ConnectionError {
                                payload: Error::new(err.message),
                            })
                            .unwrap(),
                        ))),
                        Protocols::GraphQLWS => {
                            Poll::Ready(Some(WsMessage::Close(1002, err.message)))
                        }
                    },
                };
            }
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
