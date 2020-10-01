use actix::{
    Actor, ActorContext, ActorFuture, ActorStream, AsyncContext, ContextFutureSpawner,
    StreamHandler, WrapFuture, WrapStream,
};
use actix_http::ws;
use actix_web_actors::ws::{Message, ProtocolError, WebsocketContext};
use async_graphql::http::WebSocket;
use async_graphql::{Data, ObjectType, Result, Schema, SubscriptionType};
use futures::channel::mpsc;
use futures::SinkExt;
use std::time::{Duration, Instant};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// Actor for subscription via websocket
pub struct WSSubscription<Query, Mutation, Subscription> {
    schema: Option<Schema<Query, Mutation, Subscription>>,
    last_heartbeat: Instant,
    messages: Option<mpsc::UnboundedSender<Vec<u8>>>,
    initializer: Option<Box<dyn FnOnce(serde_json::Value) -> Result<Data> + Send + Sync>>,
    continuation: Vec<u8>,
}

impl<Query, Mutation, Subscription> WSSubscription<Query, Mutation, Subscription>
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    /// Create an actor for subscription connection via websocket.
    pub fn new(schema: Schema<Query, Mutation, Subscription>) -> Self {
        Self {
            schema: Some(schema),
            last_heartbeat: Instant::now(),
            messages: None,
            initializer: None,
            continuation: Vec::new(),
        }
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

        let (tx, rx) = mpsc::unbounded();

        WebSocket::with_data(self.schema.take().unwrap(), rx, self.initializer.take())
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
            let mut sender = self.messages.as_ref().unwrap().clone();

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
