use crate::pubsub::{new_client, remove_client, PushMessage};
use actix::{
    Actor, ActorContext, ActorFuture, AsyncContext, ContextFutureSpawner, Handler,
    ResponseActFuture, Running, StreamHandler, WrapFuture,
};
use actix_web_actors::ws::{Message, ProtocolError, WebsocketContext};
use async_graphql::http::{GQLError, GQLRequest, GQLResponse};
use async_graphql::{ObjectType, QueryResult, Schema, Subscribe, SubscriptionType, Variables};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize)]
struct OperationMessage {
    #[serde(rename = "type")]
    ty: String,
    id: Option<String>,
    payload: Option<serde_json::Value>,
}

pub struct WsSession<Query, Mutation, Subscription> {
    schema: Arc<Schema<Query, Mutation, Subscription>>,
    hb: Instant,
    client_id: usize,
    subscribes: HashMap<String, Arc<Subscribe>>,
}

impl<Query, Mutation, Subscription> WsSession<Query, Mutation, Subscription>
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    pub fn new(schema: Arc<Schema<Query, Mutation, Subscription>>) -> Self {
        Self {
            schema,
            hb: Instant::now(),
            client_id: 0,
            subscribes: Default::default(),
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

        new_client(ctx.address().recipient())
            .into_actor(self)
            .then(|client_id, actor, _| {
                actor.client_id = client_id.unwrap();
                async {}.into_actor(actor)
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        remove_client(self.client_id);
        Running::Stop
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
                if let Ok(msg) = serde_json::from_str::<OperationMessage>(&s) {
                    match msg.ty.as_str() {
                        "connection_init" => {
                            ctx.text(
                                serde_json::to_string(&OperationMessage {
                                    ty: "connection_ack".to_string(),
                                    id: None,
                                    payload: None,
                                })
                                .unwrap(),
                            );
                        }
                        "start" => {
                            if let (Some(id), Some(payload)) = (msg.id, msg.payload) {
                                if let Ok(request) = serde_json::from_value::<GQLRequest>(payload) {
                                    let builder = self.schema.subscribe(&request.query);
                                    let builder = if let Some(variables) = request.variables {
                                        match Variables::parse_from_json(variables) {
                                            Ok(variables) => builder.variables(variables),
                                            Err(_) => builder,
                                        }
                                    } else {
                                        builder
                                    };
                                    let builder =
                                        if let Some(operation_name) = &request.operation_name {
                                            builder.operator_name(&operation_name)
                                        } else {
                                            builder
                                        };
                                    let subscribe = match builder.execute() {
                                        Ok(subscribe) => subscribe,
                                        Err(err) => {
                                            ctx.text(
                                                serde_json::to_string(&OperationMessage {
                                                    ty: "error".to_string(),
                                                    id: Some(id),
                                                    payload: Some(
                                                        serde_json::to_value(GQLError(&err))
                                                            .unwrap(),
                                                    ),
                                                })
                                                .unwrap(),
                                            );
                                            return;
                                        }
                                    };
                                    self.subscribes.insert(id, Arc::new(subscribe));
                                }
                            }
                        }
                        "stop" => {
                            if let Some(id) = msg.id {
                                self.subscribes.remove(&id);
                            }
                        }
                        "connection_terminate" => {
                            ctx.stop();
                        }
                        _ => {}
                    }
                }
            }
            Message::Binary(_) | Message::Close(_) | Message::Continuation(_) => {
                ctx.stop();
            }
            Message::Nop => {}
        }
    }
}

impl<Query, Mutation, Subscription> Handler<PushMessage>
    for WsSession<Query, Mutation, Subscription>
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    type Result = ResponseActFuture<Self, std::result::Result<(), ()>>;

    fn handle(&mut self, msg: PushMessage, _ctx: &mut Self::Context) -> Self::Result {
        let subscribes = self.subscribes.clone();
        let schema = self.schema.clone();
        Box::new(
            async move {
                let mut push_msgs = Vec::new();
                for (id, subscribe) in subscribes {
                    let res = match subscribe.resolve(&schema, msg.0.as_ref()).await {
                        Ok(Some(value)) => Some(Ok(value)),
                        Ok(None) => None,
                        Err(err) => Some(Err(err)),
                    };
                    if let Some(res) = res {
                        let push_msg = serde_json::to_string(&OperationMessage {
                            ty: "data".to_string(),
                            id: Some(id.clone()),
                            payload: Some(
                                serde_json::to_value(GQLResponse(res.map(|data| QueryResult {
                                    data,
                                    extensions: None,
                                })))
                                .unwrap(),
                            ),
                        })
                        .unwrap();
                        push_msgs.push(push_msg);
                    }
                }
                push_msgs
            }
            .into_actor(self)
            .map(|msgs, _, ctx| {
                for msg in msgs {
                    ctx.text(msg);
                }
                Ok(())
            }),
        )
    }
}
