use crate::{ObjectType, Result, Schema, SubscriptionType};
use bytes::Bytes;
use futures::channel::mpsc;
use futures::task::{AtomicWaker, Context, Poll};
use futures::{Stream, StreamExt};
use slab::Slab;
use std::future::Future;
use std::pin::Pin;

/// Use to hold all subscription stream for the `SubscriptionConnection`
pub struct SubscriptionStreams {
    streams: Slab<Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send>>>,
}

#[allow(missing_docs)]
impl SubscriptionStreams {
    pub fn add<S: Stream<Item = Result<serde_json::Value>> + Send + 'static>(
        &mut self,
        stream: S,
    ) -> usize {
        self.streams.insert(Box::pin(stream))
    }

    pub fn remove(&mut self, id: usize) {
        self.streams.remove(id);
    }
}

/// Subscription transport
///
/// You can customize your transport by implementing this trait.
#[async_trait::async_trait]
pub trait SubscriptionTransport: Send + Sync + Unpin + 'static {
    /// The error type.
    type Error;

    /// Parse the request data here.
    /// If you have a new subscribe, create a stream with the `Schema::create_subscription_stream`, and then call `SubscriptionStreams::add`.
    /// You can return a `Byte`, which will be sent to the client. If it returns an error, the connection will be broken.
    async fn handle_request<Query, Mutation, Subscription>(
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
    fn handle_response(&mut self, id: usize, res: Result<serde_json::Value>) -> Option<Bytes>;
}

pub fn create_connection<Query, Mutation, Subscription, T: SubscriptionTransport>(
    schema: Schema<Query, Mutation, Subscription>,
    mut transport: T,
) -> (
    mpsc::UnboundedSender<Bytes>,
    impl Stream<Item = Bytes> + Unpin,
)
where
    Query: ObjectType + Sync + Send + 'static,
    Mutation: ObjectType + Sync + Send + 'static,
    Subscription: SubscriptionType + Sync + Send + 'static,
{
    let (tx_bytes, rx_bytes) = mpsc::unbounded();
    let stream = async_stream::stream! {
        let mut streams = SubscriptionStreams {
            streams: Default::default(),
        };
        let mut inner_stream = SubscriptionStream {
            schema: &schema,
            transport: Some(&mut transport),
            streams: Some(&mut streams),
            rx_bytes,
            handle_request_fut: None,
            waker: AtomicWaker::new(),
        };
        while let Some(data) = inner_stream.next().await {
            yield data;
        }
    };
    (tx_bytes, Box::pin(stream))
}

type HandleRequestBoxFut<'a, T> = Pin<
    Box<
        dyn Future<
                Output = (
                    std::result::Result<Option<Bytes>, <T as SubscriptionTransport>::Error>,
                    &'a mut T,
                    &'a mut SubscriptionStreams,
                ),
            > + Send
            + 'a,
    >,
>;

#[allow(missing_docs)]
#[allow(clippy::type_complexity)]
struct SubscriptionStream<'a, Query, Mutation, Subscription, T: SubscriptionTransport> {
    schema: &'a Schema<Query, Mutation, Subscription>,
    transport: Option<&'a mut T>,
    streams: Option<&'a mut SubscriptionStreams>,
    rx_bytes: mpsc::UnboundedReceiver<Bytes>,
    handle_request_fut: Option<HandleRequestBoxFut<'a, T>>,
    waker: AtomicWaker,
}

impl<'a, Query, Mutation, Subscription, T> Stream
    for SubscriptionStream<'a, Query, Mutation, Subscription, T>
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
    T: SubscriptionTransport,
{
    type Item = Bytes;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = &mut *self;

        loop {
            // receive bytes
            if let Some(handle_request_fut) = &mut this.handle_request_fut {
                match handle_request_fut.as_mut().poll(cx) {
                    Poll::Ready((Ok(bytes), transport, streams)) => {
                        this.transport = Some(transport);
                        this.streams = Some(streams);
                        this.handle_request_fut = None;
                        if let Some(bytes) = bytes {
                            return Poll::Ready(Some(bytes));
                        }
                        continue;
                    }
                    Poll::Ready((Err(_), _, _)) => return Poll::Ready(None),
                    Poll::Pending => {}
                }
            } else {
                match Pin::new(&mut this.rx_bytes).poll_next(cx) {
                    Poll::Ready(Some(data)) => {
                        let transport = this.transport.take().unwrap();
                        let schema = this.schema;
                        let streams = this.streams.take().unwrap();
                        this.handle_request_fut = Some(Box::pin(async move {
                            let res = transport.handle_request(schema, streams, data).await;
                            (res, transport, streams)
                        }));
                        this.waker.wake();
                        continue;
                    }
                    Poll::Ready(None) => return Poll::Ready(None),
                    Poll::Pending => {}
                }
            }

            // receive msg
            if let (Some(streams), Some(transport)) = (&mut this.streams, &mut this.transport) {
                if !streams.streams.is_empty() {
                    let mut closed = Vec::new();

                    for (id, incoming_stream) in &mut streams.streams {
                        match incoming_stream.as_mut().poll_next(cx) {
                            Poll::Ready(Some(res)) => {
                                if res.is_err() {
                                    closed.push(id);
                                }
                                if let Some(bytes) = transport.handle_response(id, res) {
                                    return Poll::Ready(Some(bytes));
                                }
                            }
                            Poll::Ready(None) => {
                                closed.push(id);
                            }
                            Poll::Pending => {}
                        }
                    }

                    closed.iter().for_each(|id| streams.remove(*id));
                    this.waker.register(cx.waker());
                    return Poll::Pending;
                } else {
                    this.waker.register(cx.waker());
                    return Poll::Pending;
                }
            } else {
                return Poll::Pending;
            }
        }
    }
}
