use std::{future::Future, str::FromStr, sync::Arc};

use async_graphql::{
    http::{WebSocket as AGWebSocket, WebSocketProtocols, WsMessage, ALL_WEBSOCKET_PROTOCOLS},
    Data, Executor, PerMessagePostHook, PerMessagePreHook, Result,
};
use futures_util::{
    future::Ready,
    future::{self, BoxFuture},
    StreamExt,
};
use tide::Endpoint;
use tide_websockets::{tungstenite::protocol::CloseFrame, Message};

/// A GraphQL subscription endpoint builder.
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
pub struct GraphQLSubscription<E, OnConnInit> {
    executor: E,
    on_connection_init: OnConnInit,
    per_message_pre_hook: Option<Arc<PerMessagePreHook>>,
    per_message_post_hook: Option<Arc<PerMessagePostHook>>,
}

type DefaultOnConnInitType = fn(serde_json::Value) -> Ready<async_graphql::Result<Data>>;

fn default_on_connection_init(_: serde_json::Value) -> Ready<async_graphql::Result<Data>> {
    futures_util::future::ready(Ok(Data::default()))
}

impl<E> GraphQLSubscription<E, DefaultOnConnInitType>
where
    E: Executor,
{
    /// Create a [`GraphQLSubscription`] object.
    pub fn new(executor: E) -> Self {
        GraphQLSubscription {
            executor,
            on_connection_init: default_on_connection_init,
            per_message_pre_hook: None,
            per_message_post_hook: None,
        }
    }
}

impl<E, OnConnInit, OnConnInitFut> GraphQLSubscription<E, OnConnInit>
where
    E: Executor,
    OnConnInit: Fn(serde_json::Value) -> OnConnInitFut + Clone + Send + Sync + 'static,
    OnConnInitFut: Future<Output = async_graphql::Result<Data>> + Send + 'static,
{
    /// Specify a callback function to be called when the connection is
    /// initialized.
    ///
    /// You can get something from the payload of [`GQL_CONNECTION_INIT` message](https://github.com/apollographql/subscriptions-transport-ws/blob/master/PROTOCOL.md#gql_connection_init) to create [`Data`].
    /// The data returned by this callback function will be merged with the data
    /// specified by [`with_data`].
    pub fn on_connection_init<OnConnInit2, Fut>(
        self,
        callback: OnConnInit2,
    ) -> GraphQLSubscription<E, OnConnInit2>
    where
        OnConnInit2: Fn(serde_json::Value) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = async_graphql::Result<Data>> + Send + 'static,
    {
        GraphQLSubscription {
            executor: self.executor,
            on_connection_init: callback,
            per_message_pre_hook: self.per_message_pre_hook,
            per_message_post_hook: self.per_message_post_hook,
        }
    }

    /// Specify a per-message pre-hook.
    ///
    /// This hook will run for each message that the subscription stream emits, before running
    /// the resolvers. It can be used for starting a transaction, that all resolvers will use.
    #[must_use]
    pub fn per_message_pre_hook<F, Fut>(self, hook: F) -> Self
    where
        Fut: Future<Output = Result<Option<Data>>> + Send + 'static,
        F: Fn() -> Fut + Send + Sync + 'static,
    {
        GraphQLSubscription {
            executor: self.executor,
            on_connection_init: self.on_connection_init,
            per_message_pre_hook: Some(Arc::new(move || Box::pin(hook()))),
            per_message_post_hook: self.per_message_post_hook,
        }
    }

    /// Specify a per-message post-hook.
    ///
    /// This hook will run for each message that the subscription stream emits, after running
    /// the resolvers. It can be used for committing a transaction, that all resolvers will use.
    #[must_use]
    pub fn per_message_post_hook<F>(self, hook: F) -> Self
    where
        F: for<'a> Fn(&'a Data) -> BoxFuture<'a, Result<()>> + Send + Sync + 'static,
    {
        GraphQLSubscription {
            executor: self.executor,
            on_connection_init: self.on_connection_init,
            per_message_pre_hook: self.per_message_pre_hook,
            per_message_post_hook: Some(Arc::new(hook)),
        }
    }

    /// Consumes this builder to create a tide endpoint.
    pub fn build<S: Send + Sync + Clone + 'static>(self) -> impl Endpoint<S> {
        tide_websockets::WebSocket::<S, _>::new(move |request, connection| {
            let executor = self.executor.clone();
            let on_connection_init = self.on_connection_init.clone();
            let per_message_pre_hook = self.per_message_pre_hook.clone();
            let per_message_post_hook = self.per_message_post_hook.clone();
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
                .per_message_pre_hook(per_message_pre_hook)
                .per_message_post_hook(per_message_post_hook);

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
