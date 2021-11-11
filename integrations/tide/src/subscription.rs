use std::future::Future;
use std::marker::PhantomData;
use std::str::FromStr;

use async_graphql::http::{WebSocket as AGWebSocket, WebSocketProtocols, WsMessage};
use async_graphql::{Data, ObjectType, Result, Schema, SubscriptionType};
use futures_util::future::Ready;
use futures_util::{future, StreamExt};
use tide::{Endpoint, Request, StatusCode};
use tide_websockets::Message;

type DefaultOnConnCreateType<S> = fn(&Request<S>) -> Ready<Result<Data>>;

fn default_on_connection_create<S>(_: &Request<S>) -> Ready<Result<Data>> {
    futures_util::future::ready(Ok(Data::default()))
}

type DefaultOnConnInitType = fn(serde_json::Value) -> Ready<Result<Data>>;

fn default_on_connection_init(_: serde_json::Value) -> Ready<Result<Data>> {
    futures_util::future::ready(Ok(Data::default()))
}

/// GraphQL subscription builder.
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
pub struct GraphQLSubscriptionBuilder<TideState, Query, Mutation, Subscription, OnCreate, OnInit> {
    schema: Schema<Query, Mutation, Subscription>,
    on_connection_create: OnCreate,
    on_connection_init: OnInit,
    _mark: PhantomData<TideState>,
}

impl<TideState, Query, Mutation, Subscription>
    GraphQLSubscriptionBuilder<
        TideState,
        Query,
        Mutation,
        Subscription,
        DefaultOnConnCreateType<TideState>,
        DefaultOnConnInitType,
    >
{
    /// Create a GraphQL subscription builder.
    pub fn new(schema: Schema<Query, Mutation, Subscription>) -> Self {
        Self {
            schema,
            on_connection_create: default_on_connection_create,
            on_connection_init: default_on_connection_init,
            _mark: Default::default(),
        }
    }
}

impl<S, Query, Mutation, Subscription, OnCreate, OnInit>
    GraphQLSubscriptionBuilder<S, Query, Mutation, Subscription, OnCreate, OnInit>
{
    /// Specify the callback function to be called when the connection is created.
    ///
    /// You can get something from the incoming request to create [`Data`].
    pub fn on_connection_create<OnCreate2, Fut>(
        self,
        callback: OnCreate2,
    ) -> GraphQLSubscriptionBuilder<S, Query, Mutation, Subscription, OnCreate2, OnInit>
    where
        OnCreate2: Fn(&Request<S>) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = Result<Data>> + Send + 'static,
    {
        GraphQLSubscriptionBuilder {
            schema: self.schema,
            on_connection_create: callback,
            on_connection_init: self.on_connection_init,
            _mark: Default::default(),
        }
    }

    /// Specify a callback function to be called when the connection is initialized.
    ///
    /// You can get something from the payload of [`GQL_CONNECTION_INIT` message](https://github.com/apollographql/subscriptions-transport-ws/blob/master/PROTOCOL.md#gql_connection_init) to create [`Data`].
    pub fn on_connection_init<OnInit2, Fut>(
        self,
        callback: OnInit2,
    ) -> GraphQLSubscriptionBuilder<S, Query, Mutation, Subscription, OnCreate, OnInit2>
    where
        OnInit2: FnOnce(serde_json::Value) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = Result<Data>> + Send + 'static,
    {
        GraphQLSubscriptionBuilder {
            schema: self.schema,
            on_connection_create: self.on_connection_create,
            on_connection_init: callback,
            _mark: Default::default(),
        }
    }
}

impl<TideState, Query, Mutation, Subscription, OnCreate, OnCreateFut, OnInit, OnInitFut>
    GraphQLSubscriptionBuilder<TideState, Query, Mutation, Subscription, OnCreate, OnInit>
where
    TideState: Send + Sync + Clone + 'static,
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
    OnCreate: Fn(&Request<TideState>) -> OnCreateFut + Send + Clone + Sync + 'static,
    OnCreateFut: Future<Output = async_graphql::Result<Data>> + Send + 'static,
    OnInit: FnOnce(serde_json::Value) -> OnInitFut + Clone + Send + Sync + 'static,
    OnInitFut: Future<Output = async_graphql::Result<Data>> + Send + 'static,
{
    /// Create an endpoint for graphql subscription.
    pub fn build(self) -> impl Endpoint<TideState> {
        tide_websockets::WebSocket::<TideState, _>::new(move |request, connection| {
            let schema = self.schema.clone();
            let on_connection_create = self.on_connection_create.clone();
            let on_connection_init = self.on_connection_init.clone();

            async move {
                let data = on_connection_create(&request)
                    .await
                    .map_err(|_| tide::Error::from_str(StatusCode::BadRequest, "bad request"))?;

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
                .connection_data(data)
                .on_connection_init(on_connection_init);

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
        .with_protocols(&["graphql-transport-ws", "graphql-ws"])
    }
}
