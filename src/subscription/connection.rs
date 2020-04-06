use crate::{ObjectType, Schema, SubscriptionType};
use bytes::Bytes;
use futures::channel::mpsc;
use futures::task::{Context, Poll};
use futures::Stream;
use slab::Slab;
use std::collections::VecDeque;
use std::pin::Pin;

/// Use to hold all subscription stream for the `SubscriptionConnection`
pub struct SubscriptionStreams {
    streams: Slab<Pin<Box<dyn Stream<Item = serde_json::Value>>>>,
}

#[allow(missing_docs)]
impl SubscriptionStreams {
    pub fn add(&mut self, stream: Pin<Box<dyn Stream<Item = serde_json::Value>>>) -> usize {
        self.streams.insert(stream)
    }

    pub fn remove(&mut self, id: usize) {
        self.streams.remove(id);
    }
}

/// Subscription transport
///
/// You can customize your transport by implementing this trait.
pub trait SubscriptionTransport: Send + Sync + Unpin + 'static {
    /// The error type.
    type Error;

    /// Parse the request data here.
    /// If you have a new subscribe, create a stream with the `Schema::create_subscription_stream`, and then call `SubscriptionStreams::add`.
    /// You can return a `Byte`, which will be sent to the client. If it returns an error, the connection will be broken.
    fn handle_request<Query, Mutation, Subscription>(
        &mut self,
        schema: &Schema<Query, Mutation, Subscription>,
        streams: &mut SubscriptionStreams,
        data: Bytes,
    ) -> std::result::Result<Option<Bytes>, Self::Error>
    where
        Query: ObjectType + Sync + Send + 'static,
        Mutation: ObjectType + Sync + Send + 'static,
        Subscription: SubscriptionType + Sync + Send + 'static;

    /// When a response message is generated, you can convert the message to the format you want here.
    fn handle_response(&mut self, id: usize, value: serde_json::Value) -> Option<Bytes>;
}

pub fn create_connection<Query, Mutation, Subscription, T: SubscriptionTransport>(
    schema: Schema<Query, Mutation, Subscription>,
    transport: T,
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
    (
        tx_bytes.clone(),
        SubscriptionStream {
            schema,
            transport,
            streams: SubscriptionStreams {
                streams: Default::default(),
            },
            rx_bytes,
            send_queue: VecDeque::new(),
        },
    )
}

#[allow(missing_docs)]
pub struct SubscriptionStream<Query, Mutation, Subscription, T: SubscriptionTransport> {
    schema: Schema<Query, Mutation, Subscription>,
    transport: T,
    streams: SubscriptionStreams,
    rx_bytes: mpsc::Receiver<Bytes>,
    send_queue: VecDeque<Bytes>,
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
                        .handle_request(&this.schema, &mut this.streams, data)
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

            // receive msg
            let this = &mut *self;
            if !this.streams.streams.is_empty() {
                loop {
                    let mut num_closed = 0;
                    let mut num_pending = 0;

                    for (id, incoming_stream) in &mut this.streams.streams {
                        match incoming_stream.as_mut().poll_next(cx) {
                            Poll::Ready(Some(value)) => {
                                if let Some(bytes) = this.transport.handle_response(id, value) {
                                    this.send_queue.push_back(bytes);
                                }
                            }
                            Poll::Ready(None) => {
                                num_closed += 1;
                            }
                            Poll::Pending => {
                                num_pending += 1;
                            }
                        }
                    }

                    if num_closed == this.streams.streams.len() {
                        // all closed
                        return Poll::Ready(None);
                    } else if num_pending == this.streams.streams.len() {
                        return Poll::Pending;
                    }
                }
            } else {
                return Poll::Pending;
            }
        }
    }
}
