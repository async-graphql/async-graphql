use crate::BoxOnConnectFn;
use actix::{
    Actor, ActorContext, ActorFuture, AsyncContext, ContextFutureSpawner, StreamHandler, WrapFuture,
};
use actix_web::HttpRequest;
use actix_web_actors::ws::{Message, ProtocolError, WebsocketContext};
use async_graphql::{ObjectType, Schema, SubscriptionType, WebSocketTransport};
use bytes::Bytes;
use futures::channel::mpsc;
use futures::SinkExt;
use std::time::{Duration, Instant};

pub struct WsSession<Query, Mutation, Subscription> {
    req: HttpRequest,
    schema: Schema<Query, Mutation, Subscription>,
    hb: Instant,
    sink: Option<mpsc::Sender<Bytes>>,
    on_connect: Option<BoxOnConnectFn<Query, Mutation, Subscription>>,
}

impl<Query, Mutation, Subscription> WsSession<Query, Mutation, Subscription>
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    pub fn new(
        schema: Schema<Query, Mutation, Subscription>,
        req: HttpRequest,
        on_connect: Option<BoxOnConnectFn<Query, Mutation, Subscription>>,
    ) -> Self {
        Self {
            req,
            schema,
            hb: Instant::now(),
            sink: None,
            on_connect,
        }
    }

    fn hb(&self, ctx: &mut WebsocketContext<Self>) {
        ctx.run_interval(Duration::new(1, 0), |act, ctx| {
            if Instant::now().duration_since(act.hb) > Duration::new(10, 0) {
                ctx.stop();
            }
        });
    }
}

impl<Query, Mutation, Subscription> Actor for WsSession<Query, Mutation, Subscription>
where
    Query: ObjectType + Sync + Send + 'static,
    Mutation: ObjectType + Sync + Send + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        let schema = self.schema.clone();
        let on_connect = self.on_connect.clone();
        let req = self.req.clone();
        async move {
            let mut builder = schema
                .clone()
                .subscription_connection(WebSocketTransport::default());
            if let Some(on_connect) = on_connect {
                builder = on_connect(&req, builder);
            }
            builder.build().await
        }
        .into_actor(self)
        .then(|(sink, stream), actor, ctx| {
            actor.sink = Some(sink);
            ctx.add_stream(stream);
            async {}.into_actor(actor)
        })
        .wait(ctx);
    }
}

impl<Query, Mutation, Subscription> StreamHandler<Result<Message, ProtocolError>>
    for WsSession<Query, Mutation, Subscription>
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
    for WsSession<Query, Mutation, Subscription>
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    fn handle(&mut self, data: Bytes, ctx: &mut Self::Context) {
        ctx.text(unsafe { std::str::from_utf8_unchecked(&data) });
    }
}
