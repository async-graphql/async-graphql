use crate::context::Data;
use crate::schema::SUBSCRIPTION_SENDERS;
use crate::subscription::SubscriptionStub;
use crate::{ObjectType, Result, Schema, SubscriptionType};
use bytes::Bytes;
use futures::channel::mpsc;
use futures::task::{Context, Poll};
use futures::{Future, FutureExt, Stream};
use slab::Slab;
use std::any::Any;
use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::Arc;

/// Subscription stubs, use to hold all subscription information for the `SubscriptionConnection`
pub struct SubscriptionStubs<Query, Mutation, Subscription> {
    stubs: Slab<SubscriptionStub<Query, Mutation, Subscription>>,
    ctx_data: Option<Arc<Data>>,
}

#[allow(missing_docs)]
impl<Query, Mutation, Subscription> SubscriptionStubs<Query, Mutation, Subscription> {
    pub fn add(&mut self, mut stub: SubscriptionStub<Query, Mutation, Subscription>) -> usize {
        stub.ctx_data = self.ctx_data.clone();
        self.stubs.insert(stub)
    }

    pub fn remove(&mut self, id: usize) {
        self.stubs.remove(id);
    }
}

/// Subscription transport
///
/// You can customize your transport by implementing this trait.
pub trait SubscriptionTransport: Send + Sync + Unpin + 'static {
    /// The error type.
    type Error;

    /// Parse the request data here.
    /// If you have a new request, create a `SubscriptionStub` with the `Schema::create_subscription_stub`, and then call `SubscriptionStubs::add`.
    /// You can return a `Byte`, which will be sent to the client. If it returns an error, the connection will be broken.
    fn handle_request<Query, Mutation, Subscription>(
        &mut self,
        schema: &Schema<Query, Mutation, Subscription>,
        stubs: &mut SubscriptionStubs<Query, Mutation, Subscription>,
        data: Bytes,
    ) -> std::result::Result<Option<Bytes>, Self::Error>
    where
        Query: ObjectType + Sync + Send + 'static,
        Mutation: ObjectType + Sync + Send + 'static,
        Subscription: SubscriptionType + Sync + Send + 'static;

    /// When a response message is generated, you can convert the message to the format you want here.
    fn handle_response(&mut self, id: usize, result: Result<serde_json::Value>) -> Option<Bytes>;
}

pub async fn create_connection<Query, Mutation, Subscription, T: SubscriptionTransport>(
    schema: Schema<Query, Mutation, Subscription>,
    transport: T,
    ctx_data: Option<Data>,
) -> (
    mpsc::Sender<Bytes>,
    SubscriptionStream<Query, Mutation, Subscription, T>,
)
where
    Query: ObjectType + Sync + Send + 'static,
    Mutation: ObjectType + Sync + Send + 'static,
    Subscription: SubscriptionType + Sync + Send + 'static,
{
    let (tx_bytes, rx_bytes) = mpsc::channel(8);
    let (tx_msg, rx_msg) = mpsc::channel(8);
    let mut senders = SUBSCRIPTION_SENDERS.lock().await;
    senders.insert(tx_msg);
    (
        tx_bytes.clone(),
        SubscriptionStream {
            schema,
            transport,
            stubs: SubscriptionStubs {
                stubs: Default::default(),
                ctx_data: ctx_data.map(Arc::new),
            },
            rx_bytes,
            rx_msg,
            send_queue: VecDeque::new(),
            resolve_queue: VecDeque::default(),
            resolve_fut: None,
        },
    )
}

#[allow(missing_docs)]
pub struct SubscriptionStream<Query, Mutation, Subscription, T: SubscriptionTransport> {
    schema: Schema<Query, Mutation, Subscription>,
    transport: T,
    stubs: SubscriptionStubs<Query, Mutation, Subscription>,
    rx_bytes: mpsc::Receiver<Bytes>,
    rx_msg: mpsc::Receiver<Arc<dyn Any + Sync + Send>>,
    send_queue: VecDeque<Bytes>,
    resolve_queue: VecDeque<Arc<dyn Any + Sync + Send>>,
    resolve_fut: Option<Pin<Box<dyn Future<Output = ()>>>>,
}

impl<Query, Mutation, Subscription, T> Stream
    for SubscriptionStream<Query, Mutation, Subscription, T>
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
    T: SubscriptionTransport,
{
    type Item = Bytes;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            // send bytes
            if let Some(bytes) = self.send_queue.pop_front() {
                return Poll::Ready(Some(bytes));
            }

            // receive bytes
            match Pin::new(&mut self.rx_bytes).poll_next(cx) {
                Poll::Ready(Some(data)) => {
                    let this = &mut *self;
                    match this
                        .transport
                        .handle_request(&this.schema, &mut this.stubs, data)
                    {
                        Ok(Some(bytes)) => {
                            this.send_queue.push_back(bytes);
                            continue;
                        }
                        Ok(None) => {}
                        Err(_) => return Poll::Ready(None),
                    }
                }
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => {}
            }

            if let Some(resolve_fut) = &mut self.resolve_fut {
                match resolve_fut.poll_unpin(cx) {
                    Poll::Ready(_) => {
                        self.resolve_fut = None;
                    }
                    Poll::Pending => return Poll::Pending,
                }
            } else if let Some(msg) = self.resolve_queue.pop_front() {
                // FIXME: I think this code is safe, but I don't know how to implement it in safe code.
                let this = &mut *self;
                let stubs = &this.stubs as *const SubscriptionStubs<Query, Mutation, Subscription>;
                let transport = &mut this.transport as *mut T;
                let send_queue = &mut this.send_queue as *mut VecDeque<Bytes>;
                let fut = async move {
                    unsafe {
                        for (id, stub) in (*stubs).stubs.iter() {
                            if let Some(res) = stub.resolve(msg.as_ref()).await.transpose() {
                                if let Some(bytes) = (*transport).handle_response(id, res) {
                                    (*send_queue).push_back(bytes);
                                }
                            }
                        }
                    }
                };
                self.resolve_fut = Some(Box::pin(fut));
                continue;
            }

            // receive msg
            match Pin::new(&mut self.rx_msg).poll_next(cx) {
                Poll::Ready(Some(msg)) => {
                    self.resolve_queue.push_back(msg);
                }
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => {
                    // all pending
                    return Poll::Pending;
                }
            }
        }
    }
}
