use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;

use async_graphql::http::{WebSocket as AGWebSocket, WebSocketProtocols, WsMessage};
use async_graphql::{Data, ObjectType, Result, Schema, SubscriptionType};
use futures_util::{future, StreamExt};
use tide::{Endpoint, Request, Response};
use tide_websockets::Message;

/// GraphQL subscription endpoint.
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
pub struct Subscription<S> {
    inner: Pin<Box<dyn Endpoint<S>>>,
}

#[async_trait::async_trait]
impl<S> Endpoint<S> for Subscription<S>
where
    S: Send + Sync + Clone + 'static,
{
    async fn call(&self, req: Request<S>) -> tide::Result<Response> {
        self.inner.call(req).await
    }
}

impl<S> Subscription<S>
where
    S: Send + Sync + Clone + 'static,
{
    /// Create a graphql subscription endpoint.
    pub fn new<Query, Mutation, Subscription>(schema: Schema<Query, Mutation, Subscription>) -> Self
    where
        Query: ObjectType + 'static,
        Mutation: ObjectType + 'static,
        Subscription: SubscriptionType + 'static,
    {
        Self::new_with_initializer(schema, |_| {
            futures_util::future::ready(Ok(Default::default()))
        })
    }

    /// Create a graphql subscription endpoint.
    ///
    /// Specifies that a function converts the init payload to data.
    pub fn new_with_initializer<Query, Mutation, Subscription, F, R>(
        schema: Schema<Query, Mutation, Subscription>,
        initializer: F,
    ) -> Self
    where
        Query: ObjectType + 'static,
        Mutation: ObjectType + 'static,
        Subscription: SubscriptionType + 'static,
        F: FnOnce(serde_json::Value) -> R + Unpin + Send + Sync + Clone + 'static,
        R: Future<Output = Result<Data>> + Send + 'static,
    {
        let endpoint = tide_websockets::WebSocket::<S, _>::new(move |request, connection| {
            let schema = schema.clone();
            let initializer = initializer.clone();
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
                let mut stream = AGWebSocket::with_data(
                    schema.clone(),
                    connection
                        .take_while(|msg| future::ready(msg.is_ok()))
                        .map(Result::unwrap)
                        .map(Message::into_data),
                    initializer,
                    protocol,
                );
                while let Some(data) = stream.next().await {
                    match data {
                        WsMessage::Text(text) => {
                            if sink.send_string(text).await.is_err() {
                                break;
                            }
                        }
                        WsMessage::Close(_code, _msg) => {
                            // TODO: Send close frame
                            break;
                        }
                    }
                }

                Ok(())
            }
        })
        .with_protocols(&["graphql-transport-ws", "graphql-ws"]);
        Self {
            inner: Box::pin(endpoint),
        }
    }
}
