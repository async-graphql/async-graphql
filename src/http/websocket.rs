//! WebSocket transport for subscription

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use futures_util::future::{BoxFuture, Ready};
use futures_util::stream::Stream;
use futures_util::FutureExt;
use pin_project_lite::pin_project;
use serde::{Deserialize, Serialize};

use crate::{Data, Error, ObjectType, Request, Response, Result, Schema, SubscriptionType};

pin_project! {
    /// A GraphQL connection over websocket.
    ///
    /// [Reference](https://github.com/apollographql/subscriptions-transport-ws/blob/master/PROTOCOL.md).
    pub struct WebSocket<S, F, Query, Mutation, Subscription> {
        data_initializer: Option<F>,
        init_fut: Option<BoxFuture<'static, Result<Data>>>,
        data: Arc<Data>,
        schema: Schema<Query, Mutation, Subscription>,
        streams: HashMap<String, Pin<Box<dyn Stream<Item = Response> + Send>>>,
        #[pin]
        stream: S,
        protocol: Protocols,
    }
}

impl<S, Query, Mutation, Subscription>
    WebSocket<S, fn(serde_json::Value) -> Ready<Result<Data>>, Query, Mutation, Subscription>
{
    /// Create a new websocket.
    #[must_use]
    pub fn new(
        schema: Schema<Query, Mutation, Subscription>,
        stream: S,
        protocol: Protocols,
    ) -> Self {
        Self {
            data_initializer: Some(|_| futures_util::future::ready(Ok(Default::default()))),
            init_fut: None,
            data: Arc::default(),
            schema,
            streams: HashMap::new(),
            stream,
            protocol,
        }
    }
}

impl<S, F, Query, Mutation, Subscription> WebSocket<S, F, Query, Mutation, Subscription> {
    /// Create a new websocket with a data initialization function.
    ///
    /// This function, if present, will be called with the data sent by the client in the
    /// [`GQL_CONNECTION_INIT` message](https://github.com/apollographql/subscriptions-transport-ws/blob/master/PROTOCOL.md#gql_connection_init).
    /// From that point on the returned data will be accessible to all requests.
    #[must_use]
    pub fn with_data(
        schema: Schema<Query, Mutation, Subscription>,
        stream: S,
        data_initializer: F,
        protocol: Protocols,
    ) -> Self {
        Self {
            data_initializer: Some(data_initializer),
            init_fut: None,
            data: Arc::default(),
            schema,
            streams: HashMap::new(),
            stream,
            protocol,
        }
    }
}

impl<S, F, R, Query, Mutation, Subscription> Stream
    for WebSocket<S, F, Query, Mutation, Subscription>
where
    S: Stream,
    S::Item: AsRef<[u8]>,
    F: FnOnce(serde_json::Value) -> R + Send + 'static,
    R: Future<Output = Result<Data>> + Send + 'static,
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
{
    type Item = String;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        while let Poll::Ready(message) = Pin::new(&mut this.stream).poll_next(cx) {
            let message = match message {
                Some(message) => message,
                None => return Poll::Ready(None),
            };

            let message: ClientMessage = match serde_json::from_slice(message.as_ref()) {
                Ok(message) => message,
                Err(e) => {
                    return Poll::Ready(Some(
                        serde_json::to_string(&ServerMessage::ConnectionError {
                            payload: Error::new(e.to_string()),
                        })
                        .unwrap(),
                    ))
                }
            };

            match message {
                ClientMessage::ConnectionInit { payload } => {
                    if let Some(data_initializer) = this.data_initializer.take() {
                        *this.init_fut = Some(Box::pin(async move {
                            data_initializer(payload.unwrap_or_default()).await
                        }));
                    }
                }
                ClientMessage::Start {
                    id,
                    payload: request,
                } => {
                    this.streams.insert(
                        id,
                        Box::pin(
                            this.schema
                                .execute_stream_with_ctx_data(request, Arc::clone(this.data)),
                        ),
                    );
                }
                ClientMessage::Stop { id } => {
                    if this.streams.remove(id).is_some() {
                        return Poll::Ready(Some(
                            serde_json::to_string(&ServerMessage::Complete { id }).unwrap(),
                        ));
                    }
                }
                // Note: in the revised `graphql-ws` spec, there is no equivalent to the
                // `CONNECTION_TERMINATE` `client -> server` message; rather, disconnection is
                // handled by disconnecting the websocket
                ClientMessage::ConnectionTerminate => return Poll::Ready(None),
            }
        }

        if let Some(init_fut) = this.init_fut {
            if let Poll::Ready(res) = init_fut.poll_unpin(cx) {
                *this.init_fut = None;
                return match res {
                    Ok(data) => {
                        *this.data = Arc::new(data);
                        Poll::Ready(Some(
                            serde_json::to_string(&ServerMessage::ConnectionAck).unwrap(),
                        ))
                    }
                    Err(err) => Poll::Ready(Some(
                        serde_json::to_string(&ServerMessage::ConnectionError { payload: err })
                            .unwrap(),
                    )),
                };
            }
        }

        for (id, stream) in &mut *this.streams {
            match Pin::new(stream).poll_next(cx) {
                Poll::Ready(Some(payload)) => {
                    return Poll::Ready(Some(
                        serde_json::to_string(&this.protocol.next_message(id, payload)).unwrap(),
                    ));
                }
                Poll::Ready(None) => {
                    let id = id.clone();
                    this.streams.remove(&id);
                    return Poll::Ready(Some(
                        serde_json::to_string(&ServerMessage::Complete { id: &id }).unwrap(),
                    ));
                }
                Poll::Pending => {}
            }
        }

        Poll::Pending
    }
}

/// Specification of which GraphQL Over WebSockets protocol is being utilized
#[derive(Copy, Clone)]
pub enum Protocols {
    /// [subscriptions-transport-ws protocol](https://github.com/apollographql/subscriptions-transport-ws/blob/master/PROTOCOL.md).
    SubscriptionsTransportWS,
    /// [graphql-ws protocol](https://github.com/enisdenjo/graphql-ws/blob/master/PROTOCOL.md).
    GraphQLWS,
}

impl Protocols {
    /// Returns the `Sec-WebSocket-Protocol` header value for the protocol
    pub fn sec_websocket_protocol(&self) -> &str {
        match self {
            Protocols::SubscriptionsTransportWS => "graphql-ws",
            Protocols::GraphQLWS => "graphql-transport-ws",
        }
    }

    #[inline]
    fn next_message<'s>(&self, id: &'s str, payload: Response) -> ServerMessage<'s> {
        match self {
            Protocols::SubscriptionsTransportWS => ServerMessage::Data {
                id,
                payload: Box::new(payload),
            },
            Protocols::GraphQLWS => ServerMessage::Next {
                id,
                payload: Box::new(payload),
            },
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

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientMessage<'a> {
    ConnectionInit {
        payload: Option<serde_json::Value>,
    },
    #[serde(alias = "subscribe")]
    Start {
        id: String,
        payload: Request,
    },
    #[serde(alias = "complete")]
    Stop {
        id: &'a str,
    },
    ConnectionTerminate,
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
        payload: Box<Response>,
    },
    /// graphql-ws protocol next payload
    Next {
        id: &'a str,
        payload: Box<Response>,
    },
    // Not used by this library, as it's not necessary to send
    // Error {
    //     id: &'a str,
    //     payload: serde_json::Value,
    // },
    Complete {
        id: &'a str,
    },
    // Not used by this library
    // #[serde(rename = "ka")]
    // KeepAlive
}
