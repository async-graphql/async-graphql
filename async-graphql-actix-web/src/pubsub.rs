use actix::{Actor, Context, Handler, Recipient, Supervised, SystemService};
use async_graphql::Result;
use slab::Slab;
use std::any::Any;
use std::sync::Arc;

#[derive(Message)]
#[rtype(result = "std::result::Result<(), ()>")]
pub struct PushMessage(pub Arc<dyn Any + Sync + Send>);

#[derive(Message)]
#[rtype(result = "usize")]
struct NewClient {
    recipient: Recipient<PushMessage>,
}

#[derive(Message)]
#[rtype(result = "()")]
struct RemoveClient {
    id: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
struct PubMessage(Arc<dyn Any + Sync + Send>);

struct ClientInfo {
    recipient: Recipient<PushMessage>,
}

#[derive(Default)]
struct PubSubService {
    clients: Slab<ClientInfo>,
}

impl Actor for PubSubService {
    type Context = Context<Self>;
}

impl Handler<NewClient> for PubSubService {
    type Result = usize;

    fn handle(&mut self, msg: NewClient, _ctx: &mut Context<Self>) -> Self::Result {
        self.clients.insert(ClientInfo {
            recipient: msg.recipient,
        })
    }
}

impl Handler<RemoveClient> for PubSubService {
    type Result = ();

    fn handle(&mut self, msg: RemoveClient, _ctx: &mut Context<Self>) -> Self::Result {
        self.clients.remove(msg.id);
    }
}

impl Handler<PubMessage> for PubSubService {
    type Result = ();

    fn handle(&mut self, msg: PubMessage, _ctx: &mut Context<Self>) -> Self::Result {
        for (_, client) in &self.clients {
            client.recipient.do_send(PushMessage(msg.0.clone())).ok();
        }
    }
}

impl Supervised for PubSubService {}

impl SystemService for PubSubService {}

pub async fn new_client(recipient: Recipient<PushMessage>) -> Result<usize> {
    let id = PubSubService::from_registry()
        .send(NewClient { recipient })
        .await?;
    Ok(id)
}

pub fn remove_client(id: usize) {
    PubSubService::from_registry().do_send(RemoveClient { id });
}

/// Publish a message that will be pushed to all subscribed clients.
pub fn publish_message<T: Any + Send + Sync + Sized>(msg: T) {
    PubSubService::from_registry().do_send(PubMessage(Arc::new(msg)));
}
