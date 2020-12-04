use std::str::FromStr;
use std::time::{Duration, Instant};

use actix::{
    Actor, ActorContext, ActorFuture, ActorStream, AsyncContext, ContextFutureSpawner,
    StreamHandler, WrapFuture, WrapStream,
};
use actix_http::error::PayloadError;
use actix_http::{ws, Error};
use actix_web::web::Bytes;
use actix_web::{HttpRequest, HttpResponse};
use actix_web_actors::ws::{Message, ProtocolError, WebsocketContext};
use async_graphql::http::{WebSocket, WebSocketProtocols};
use async_graphql::{Data, ObjectType, Result, Schema, SubscriptionType};
use futures_util::stream::Stream;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// Actor for subscription via websocket
pub struct WSSubscription<Query, Mutation, Subscription> {
    schema: Schema<Query, Mutation, Subscription>,
    protocol: WebSocketProtocols,
    last_heartbeat: Instant,
    messages: Option<async_channel::Sender<Vec<u8>>>,
    initializer: Option<Box<dyn FnOnce(serde_json::Value) -> Result<Data> + Send + Sync>>,
    continuation: Vec<u8>,
}

impl<Query, Mutation, Subscription> WSSubscription<Query, Mutation, Subscription>
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    /// Start an actor for subscription connection via websocket.
    pub fn start<T>(
        schema: Schema<Query, Mutation, Subscription>,
        request: &HttpRequest,
        stream: T,
    ) -> Result<HttpResponse, Error>
    where
        T: Stream<Item = Result<Bytes, PayloadError>> + 'static,
    {
        let protocol = match request
            .headers()
            .get("sec-websocket-protocol")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| WebSocketProtocols::from_str(value).ok())
        {
            Some(protocol) => protocol,
            None => {
                // default to the prior standard
                WebSocketProtocols::SubscriptionsTransportWS
            }
        };

        actix_web_actors::ws::start_with_protocols(
            Self {
                schema,
                protocol,
                last_heartbeat: Instant::now(),
                messages: None,
                initializer: None,
                continuation: Vec::new(),
            },
            &["graphql-transport-ws", "graphql-ws"],
            request,
            stream,
        )
    }

    /// Set a context data initialization function.
    pub fn initializer<F>(self, f: F) -> Self
    where
        F: FnOnce(serde_json::Value) -> Result<Data> + Send + Sync + 'static,
    {
        Self {
            initializer: Some(Box::new(f)),
            ..self
        }
    }

    fn send_heartbeats(&self, ctx: &mut WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.last_heartbeat) > CLIENT_TIMEOUT {
                ctx.stop();
            }
            ctx.ping(b"");
        });
    }
}

impl<Query, Mutation, Subscription> Actor for WSSubscription<Query, Mutation, Subscription>
where
    Query: ObjectType + Sync + Send + 'static,
    Mutation: ObjectType + Sync + Send + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.send_heartbeats(ctx);

        let (tx, rx) = async_channel::unbounded();

        WebSocket::with_data(
            self.schema.clone(),
            rx,
            self.initializer.take(),
            self.protocol,
        )
        .into_actor(self)
        .map(|response, _act, ctx| {
            ctx.text(response);
        })
        .finish()
        .spawn(ctx);

        self.messages = Some(tx);
    }
}

impl<Query, Mutation, Subscription> StreamHandler<Result<Message, ProtocolError>>
    for WSSubscription<Query, Mutation, Subscription>
where
    Query: ObjectType + Sync + Send + 'static,
    Mutation: ObjectType + Sync + Send + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
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
            Message::Text(s) => Some(s.into_bytes()),
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
