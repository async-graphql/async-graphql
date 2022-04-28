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
    http::{WebSocket, WebSocketProtocols, WsMessage, ALL_WEBSOCKET_PROTOCOLS},
    Data, ObjectType, Result, Schema, SubscriptionType,
};
use futures_util::{future::Ready, stream::Stream};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(thiserror::Error, Debug)]
#[error("failed to parse graphql protocol")]
pub struct ParseGraphQLProtocolError;

type DefaultOnConnInitType = fn(serde_json::Value) -> Ready<async_graphql::Result<Data>>;

fn default_on_connection_init(_: serde_json::Value) -> Ready<async_graphql::Result<Data>> {
    futures_util::future::ready(Ok(Data::default()))
}

/// A builder for websocket subscription actor.
pub struct GraphQLSubscription<Query, Mutation, Subscription, OnInit> {
    schema: Schema<Query, Mutation, Subscription>,
    data: Data,
    on_connection_init: OnInit,
}

impl<Query, Mutation, Subscription>
    GraphQLSubscription<Query, Mutation, Subscription, DefaultOnConnInitType>
where
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
{
    /// Create a GraphQL subscription builder.
    pub fn new(schema: Schema<Query, Mutation, Subscription>) -> Self {
        Self {
            schema,
            data: Default::default(),
            on_connection_init: default_on_connection_init,
        }
    }
}

impl<Query, Mutation, Subscription, OnInit, OnInitFut>
    GraphQLSubscription<Query, Mutation, Subscription, OnInit>
where
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
    OnInit: Fn(serde_json::Value) -> OnInitFut + Unpin + Send + 'static,
    OnInitFut: Future<Output = async_graphql::Result<Data>> + Send + 'static,
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
    pub fn on_connection_init<OnConnInit2, Fut>(
        self,
        callback: OnConnInit2,
    ) -> GraphQLSubscription<Query, Mutation, Subscription, OnConnInit2>
    where
        OnConnInit2: Fn(serde_json::Value) -> Fut + Unpin + Send + 'static,
        Fut: Future<Output = async_graphql::Result<Data>> + Send + 'static,
    {
        GraphQLSubscription {
            schema: self.schema,
            data: self.data,
            on_connection_init: callback,
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
            schema: self.schema,
            data: Some(self.data),
            protocol,
            last_heartbeat: Instant::now(),
            messages: None,
            on_connection_init: Some(self.on_connection_init),
            continuation: Vec::new(),
        };

        actix_web_actors::ws::WsResponseBuilder::new(actor, request, stream)
            .protocols(&ALL_WEBSOCKET_PROTOCOLS)
            .start()
    }
}

struct GraphQLSubscriptionActor<Query, Mutation, Subscription, OnInit> {
    schema: Schema<Query, Mutation, Subscription>,
    data: Option<Data>,
    protocol: WebSocketProtocols,
    last_heartbeat: Instant,
    messages: Option<async_channel::Sender<Vec<u8>>>,
    on_connection_init: Option<OnInit>,
    continuation: Vec<u8>,
}

impl<Query, Mutation, Subscription, OnInit, OnInitFut>
    GraphQLSubscriptionActor<Query, Mutation, Subscription, OnInit>
where
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
    OnInit: FnOnce(serde_json::Value) -> OnInitFut + Unpin + Send + 'static,
    OnInitFut: Future<Output = Result<Data>> + Send + 'static,
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

impl<Query, Mutation, Subscription, OnInit, OnInitFut> Actor
    for GraphQLSubscriptionActor<Query, Mutation, Subscription, OnInit>
where
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
    OnInit: FnOnce(serde_json::Value) -> OnInitFut + Unpin + Send + 'static,
    OnInitFut: Future<Output = Result<Data>> + Send + 'static,
{
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.send_heartbeats(ctx);

        let (tx, rx) = async_channel::unbounded();

        WebSocket::new(self.schema.clone(), rx, self.protocol)
            .connection_data(self.data.take().unwrap())
            .on_connection_init(self.on_connection_init.take().unwrap())
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

impl<Query, Mutation, Subscription, OnInit, OnInitFut> StreamHandler<Result<Message, ProtocolError>>
    for GraphQLSubscriptionActor<Query, Mutation, Subscription, OnInit>
where
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
    OnInit: FnOnce(serde_json::Value) -> OnInitFut + Unpin + Send + 'static,
    OnInitFut: Future<Output = Result<Data>> + Send + 'static,
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
