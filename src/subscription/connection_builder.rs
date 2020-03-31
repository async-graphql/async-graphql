use crate::context::Data;
use crate::subscription::create_connection;
use crate::{ObjectType, Schema, SubscriptionStream, SubscriptionTransport, SubscriptionType};
use bytes::Bytes;
use futures::channel::mpsc;
use std::any::Any;

/// SubscriptionConnection builder
pub struct SubscriptionConnectionBuilder<Query, Mutation, Subscription, T: SubscriptionTransport> {
    pub(crate) schema: Schema<Query, Mutation, Subscription>,
    pub(crate) transport: T,
    pub(crate) ctx_data: Option<Data>,
}

impl<Query, Mutation, Subscription, T: SubscriptionTransport>
    SubscriptionConnectionBuilder<Query, Mutation, Subscription, T>
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    /// Add a context data that can be accessed in the `Context`, you access it with `Context::data`.
    pub fn data<D: Any + Send + Sync>(mut self, data: D) -> Self {
        if let Some(ctx_data) = &mut self.ctx_data {
            ctx_data.insert(data);
        } else {
            let mut ctx_data = Data::default();
            ctx_data.insert(data);
            self.ctx_data = Some(ctx_data);
        }
        self
    }

    /// Create subscription connection, returns `Sink` and `Stream`.
    pub async fn build(
        self,
    ) -> (
        mpsc::Sender<Bytes>,
        SubscriptionStream<Query, Mutation, Subscription, T>,
    ) {
        create_connection(self.schema, self.transport, self.ctx_data).await
    }
}
