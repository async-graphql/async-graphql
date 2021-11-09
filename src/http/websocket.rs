//! WebSocket transport for subscription

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use futures_util::stream::Stream;
use futures_util::FutureExt;
use futures_util::{future::BoxFuture, StreamExt};
use pin_project_lite::pin_project;
use serde::{Deserialize, Serialize};

use crate::{Data, Error, ObjectType, Request, Response, Result, Schema, SubscriptionType};

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
    /// Returns the contained [WsMessage::Text] value, consuming the `self` value.
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

    /// Returns the contained [WsMessage::Close] value, consuming the `self` value.
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

type BoxInitializer =
    Box<(dyn FnOnce(serde_json::Value) -> BoxFuture<'static, Result<Data>> + Send + 'static)>;

pin_project! {
    /// A GraphQL connection over websocket.
    ///
    /// [Reference](https://github.com/apollographql/subscriptions-transport-ws/blob/master/PROTOCOL.md).
    /// [Reference](https://github.com/enisdenjo/graphql-ws/blob/master/PROTOCOL.md).
    pub struct WebSocket<S, Query, Mutation, Subscription> {
        data_initializer: Option<BoxInitializer>,
        init_fut: Option<BoxFuture<'static, Result<Data>>>,
        connection_data: Option<Data>,
        data: Option<Arc<Data>>,
        schema: Schema<Query, Mutation, Subscription>,
        streams: HashMap<String, Pin<Box<dyn Stream<Item = Response> + Send>>>,
        #[pin]
        stream: S,
        protocol: Protocols,
    }
}

type MessageMapStream<S> =
    futures_util::stream::Map<S, fn(<S as Stream>::Item) -> serde_json::Result<ClientMessage>>;

impl<S, Query, Mutation, Subscription> WebSocket<S, Query, Mutation, Subscription>
where
    S: Stream<Item = serde_json::Result<ClientMessage>>,
{
    /// Create a new websocket from [`ClientMessage`] stream.
    pub fn from_message_stream(
        schema: Schema<Query, Mutation, Subscription>,
        stream: S,
        protocol: Protocols,
    ) -> Self {
        WebSocket {
            data_initializer: Some(Box::new(|_| Box::pin(async move { Ok(Data::default()) }))),
            init_fut: None,
            connection_data: None,
            data: None,
            schema,
            streams: HashMap::new(),
            stream,
            protocol,
        }
    }

    /// Specify a connection initializer.
    ///
    /// This function, if present, will be called with the data sent by the client in the
    /// [`GQL_CONNECTION_INIT` message](https://github.com/apollographql/subscriptions-transport-ws/blob/master/PROTOCOL.md#gql_connection_init).
    /// From that point on the returned data will be accessible to all requests.
    pub fn with_initializer<F, R>(mut self, initializer: F) -> Self
    where
        F: FnOnce(serde_json::Value) -> R + Send + 'static,
        R: Future<Output = Result<Data>> + Send + 'static,
    {
        self.data_initializer = Some(Box::new(move |value| Box::pin(initializer(value))));
        self
    }

    /// Specify a connection data.
    ///
    /// This data usually comes from HTTP requests.
    /// When the `GQL_CONNECTION_INIT` message is received, this data will be merged with the data
    /// returned by the closure specified by `with_initializer` into the final subscription context data.
    pub fn connection_data(mut self, data: Data) -> Self {
        self.connection_data = Some(data);
        self
    }
}

impl<S, Query, Mutation, Subscription> WebSocket<MessageMapStream<S>, Query, Mutation, Subscription>
where
    S: Stream,
    S::Item: AsRef<[u8]>,
{
    /// Create a new websocket from bytes stream.
    pub fn new(
        schema: Schema<Query, Mutation, Subscription>,
        stream: S,
        protocol: Protocols,
    ) -> Self
    where
        S: Stream,
        S::Item: AsRef<[u8]>,
    {
        let stream = stream
            .map(ClientMessage::from_bytes as fn(S::Item) -> serde_json::Result<ClientMessage>);
        WebSocket::from_message_stream(schema, stream, protocol)
    }
}

impl<S, Query, Mutation, Subscription> Stream for WebSocket<S, Query, Mutation, Subscription>
where
    S: Stream<Item = serde_json::Result<ClientMessage>>,
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
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
                        if let Some(data_initializer) = this.data_initializer.take() {
                            *this.init_fut = Some(Box::pin(async move {
                                data_initializer(payload.unwrap_or_default()).await
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
                                Box::pin(
                                    this.schema.execute_stream_with_session_data(request, data),
                                ),
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
        /// The GraphQL Request - this can be modified by protocol implementors to add files
        /// uploads.
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
    /// Useful for detecting failed connections, displaying latency metrics or other types of network probing.
    ///
    /// https://github.com/enisdenjo/graphql-ws/blob/master/PROTOCOL.md#ping
    Ping {
        /// Additional details about the ping.
        payload: Option<serde_json::Value>,
    },
    /// The response to the Ping message.
    ///
    /// https://github.com/enisdenjo/graphql-ws/blob/master/PROTOCOL.md#pong
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
