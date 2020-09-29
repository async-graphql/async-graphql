//! WebSocket transport for subscription

use crate::resolver_utils::ObjectType;
use crate::{Data, Error, Request, Response, Result, Schema, SubscriptionType};
use futures::Stream;
use pin_project_lite::pin_project;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

pin_project! {
    /// A GraphQL connection over websocket.
    ///
    /// [Reference](https://github.com/apollographql/subscriptions-transport-ws/blob/master/PROTOCOL.md).
    pub struct WebSocket<S, F, Query, Mutation, Subscription> {
        data_initializer: Option<F>,
        data: Arc<Data>,
        schema: Schema<Query, Mutation, Subscription>,
        streams: HashMap<String, Pin<Box<dyn Stream<Item = Response> + Send>>>,
        #[pin]
        stream: S,
    }
}

impl<S, Query, Mutation, Subscription>
    WebSocket<S, fn(serde_json::Value) -> Result<Data>, Query, Mutation, Subscription>
{
    /// Create a new websocket.
    #[must_use]
    pub fn new(schema: Schema<Query, Mutation, Subscription>, stream: S) -> Self {
        Self {
            data_initializer: None,
            data: Arc::default(),
            schema,
            streams: HashMap::new(),
            stream,
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
        data_initializer: Option<F>,
    ) -> Self {
        Self {
            data_initializer,
            data: Arc::default(),
            schema,
            streams: HashMap::new(),
            stream,
        }
    }
}

impl<S, F, Query, Mutation, Subscription> Stream for WebSocket<S, F, Query, Mutation, Subscription>
where
    S: Stream,
    S::Item: AsRef<[u8]>,
    F: FnOnce(serde_json::Value) -> Result<Data>,
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    type Item = String;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();

        match this.stream.poll_next(cx) {
            Poll::Ready(message) => {
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
                        if let Some(payload) = payload {
                            if let Some(data_initializer) = this.data_initializer.take() {
                                *this.data = Arc::new(match data_initializer(payload) {
                                    Ok(data) => data,
                                    Err(e) => {
                                        return Poll::Ready(Some(
                                            serde_json::to_string(
                                                &ServerMessage::ConnectionError { payload: e },
                                            )
                                            .unwrap(),
                                        ))
                                    }
                                });
                            }
                        }
                        return Poll::Ready(Some(
                            serde_json::to_string(&ServerMessage::ConnectionAck).unwrap(),
                        ));
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
                    ClientMessage::ConnectionTerminate => return Poll::Ready(None),
                }
            }
            Poll::Pending => {}
        }

        for (id, stream) in &mut *this.streams {
            match Pin::new(stream).poll_next(cx) {
                Poll::Ready(Some(payload)) => {
                    return Poll::Ready(Some(
                        serde_json::to_string(&ServerMessage::Data {
                            id,
                            payload: Box::new(payload),
                        })
                        .unwrap(),
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

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientMessage<'a> {
    ConnectionInit { payload: Option<serde_json::Value> },
    Start { id: String, payload: Request },
    Stop { id: &'a str },
    ConnectionTerminate,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ServerMessage<'a> {
    ConnectionError { payload: Error },
    ConnectionAck,
    Data { id: &'a str, payload: Box<Response> },
    // Not used by this library, as it's not necessary to send
    // Error {
    //     id: &'a str,
    //     payload: serde_json::Value,
    // },
    Complete { id: &'a str },
    // Not used by this library
    // #[serde(rename = "ka")]
    // KeepAlive
}
