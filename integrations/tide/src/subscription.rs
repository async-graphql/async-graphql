use std::future::Future;
use std::str::FromStr;

use async_graphql::http::{
    WebSocket as AGWebSocket, WebSocketProtocols, WsMessage, ALL_WEBSOCKET_PROTOCOLS,
};
use async_graphql::{Data, ObjectType, Result, Schema, SubscriptionType};
use futures_util::future::Ready;
use futures_util::{future, StreamExt};
use tide::Endpoint;
use tide_websockets::tungstenite::protocol::CloseFrame;
use tide_websockets::Message;

/// A GraphQL subscription endpoint builder.
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
pub struct GraphQLSubscription<Query, Mutation, Subscription, OnConnInit> {
    schema: Schema<Query, Mutation, Subscription>,
    on_connection_init: OnConnInit,
}

type DefaultOnConnInitType = fn(serde_json::Value) -> Ready<async_graphql::Result<Data>>;

fn default_on_connection_init(_: serde_json::Value) -> Ready<async_graphql::Result<Data>> {
    futures_util::future::ready(Ok(Data::default()))
}

impl<Query, Mutation, Subscription>
    GraphQLSubscription<Query, Mutation, Subscription, DefaultOnConnInitType>
where
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
{
    /// Create a [`GraphQLSubscription`] object.
    pub fn new(schema: Schema<Query, Mutation, Subscription>) -> Self {
        GraphQLSubscription {
            schema,
            on_connection_init: default_on_connection_init,
        }
    }
}

impl<Query, Mutation, Subscription, OnConnInit, OnConnInitFut>
    GraphQLSubscription<Query, Mutation, Subscription, OnConnInit>
where
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
    OnConnInit: Fn(serde_json::Value) -> OnConnInitFut + Clone + Send + Sync + 'static,
    OnConnInitFut: Future<Output = async_graphql::Result<Data>> + Send + 'static,
{
    /// Specify a callback function to be called when the connection is initialized.
    ///
    /// You can get something from the payload of [`GQL_CONNECTION_INIT` message](https://github.com/apollographql/subscriptions-transport-ws/blob/master/PROTOCOL.md#gql_connection_init) to create [`Data`].
    /// The data returned by this callback function will be merged with the data specified by [`with_data`].
    pub fn on_connection_init<OnConnInit2, Fut>(
        self,
        callback: OnConnInit2,
    ) -> GraphQLSubscription<Query, Mutation, Subscription, OnConnInit2>
    where
        OnConnInit2: Fn(serde_json::Value) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = async_graphql::Result<Data>> + Send + 'static,
    {
        GraphQLSubscription {
            schema: self.schema,
            on_connection_init: callback,
        }
    }

    /// Consumes this builder to create a tide endpoint.
    pub fn build<S: Send + Sync + Clone + 'static>(self) -> impl Endpoint<S> {
        tide_websockets::WebSocket::<S, _>::new(move |request, connection| {
            let schema = self.schema.clone();
            let on_connection_init = self.on_connection_init.clone();
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
                    schema.clone(),
                    connection
                        .take_while(|msg| future::ready(msg.is_ok()))
                        .map(Result::unwrap)
                        .map(Message::into_data),
                    protocol,
                )
                .on_connection_init(on_connection_init);

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
