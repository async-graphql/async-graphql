use actix::{
    Actor, ActorContext, ActorFuture, AsyncContext, ContextFutureSpawner, StreamHandler, WrapFuture,
};
use actix_web_actors::ws::{Message, ProtocolError, WebsocketContext};
use async_graphql::{Data, FieldResult, ObjectType, Schema, SubscriptionType, WebSocketTransport};
use bytes::Bytes;
use futures::channel::mpsc;
use futures::SinkExt;
use std::time::{Duration, Instant};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// Actor for subscription via websocket
pub struct WSSubscription<Query, Mutation, Subscription> {
    schema: Schema<Query, Mutation, Subscription>,
    hb: Instant,
    sink: Option<mpsc::Sender<Bytes>>,
    init_context_data: Option<Box<dyn Fn(serde_json::Value) -> FieldResult<Data> + Send + Sync>>,
}

impl<Query, Mutation, Subscription> WSSubscription<Query, Mutation, Subscription>
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    /// Create an actor for subscription connection via websocket.
    pub fn new(schema: &Schema<Query, Mutation, Subscription>) -> Self {
        Self {
            schema: schema.clone(),
            hb: Instant::now(),
            sink: None,
            init_context_data: None,
        }
    }

    /// Set a context data initialization function.
    pub fn init_context_data<F>(self, f: F) -> Self
    where
        F: Fn(serde_json::Value) -> FieldResult<Data> + Send + Sync + 'static,
    {
        Self {
            init_context_data: Some(Box::new(f)),
            ..self
        }
    }

    fn hb(&self, ctx: &mut WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
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
        self.hb(ctx);
        let schema = self.schema.clone();
        let (sink, stream) = schema.subscription_connection(
            if let Some(init_with_payload) = self.init_context_data.take() {
                WebSocketTransport::new(init_with_payload)
            } else {
                WebSocketTransport::default()
            },
        );
        ctx.add_stream(stream);
        self.sink = Some(sink);
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

        match msg {
            Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Message::Pong(_) => {
                self.hb = Instant::now();
            }
            Message::Text(s) => {
                if let Some(mut sink) = self.sink.clone() {
                    async move { sink.send(s.into()).await }
                        .into_actor(self)
                        .then(|_, actor, _| async {}.into_actor(actor))
                        .wait(ctx);
                }
            }
            Message::Binary(_) | Message::Close(_) | Message::Continuation(_) => {
                ctx.stop();
            }
            Message::Nop => {}
        }
    }
}

impl<Query, Mutation, Subscription> StreamHandler<Bytes>
    for WSSubscription<Query, Mutation, Subscription>
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    fn handle(&mut self, data: Bytes, ctx: &mut Self::Context) {
        if let Ok(text) = std::str::from_utf8(&data) {
            ctx.text(text);
        }
    }
}
