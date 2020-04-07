use futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use futures::task::{Context, Poll};
use futures::{Stream, StreamExt};
use once_cell::sync::OnceCell;
use serde::export::PhantomData;
use slab::Slab;
use std::any::Any;
use std::pin::Pin;
use std::sync::Mutex;

struct Senders<T>(Mutex<Slab<UnboundedSender<T>>>);

struct BrokerStream<T: Sync + Send + Clone + 'static>(usize, UnboundedReceiver<T>);

impl<T: Sync + Send + Clone + 'static> Drop for BrokerStream<T> {
    fn drop(&mut self) {
        let mut senders = SimpleBroker::<T>::senders().0.lock().unwrap();
        senders.remove(self.0);
    }
}

impl<T: Sync + Send + Clone + 'static> Stream for BrokerStream<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.1.poll_next_unpin(cx)
    }
}

/// A simple broker based on memory
pub struct SimpleBroker<T>(PhantomData<T>);

impl<T: Sync + Send + Clone + 'static> SimpleBroker<T> {
    fn senders() -> &'static Senders<T> {
        static SUBSCRIBERS: OnceCell<Box<dyn Any + Send + Sync>> = OnceCell::new();
        let instance = SUBSCRIBERS.get_or_init(|| Box::new(Senders::<T>(Mutex::new(Slab::new()))));
        instance.downcast_ref::<Senders<T>>().unwrap()
    }

    /// Publish a message that all subscription streams can receive.
    pub fn publish(msg: T) {
        let mut senders = Self::senders().0.lock().unwrap();
        for (_, sender) in senders.iter_mut() {
            sender.start_send(msg.clone()).ok();
        }
    }

    /// Subscribe to the message of the specified type and returns a `Stream`.
    pub fn subscribe() -> impl Stream<Item = T> {
        let mut senders = Self::senders().0.lock().unwrap();
        let (tx, rx) = mpsc::unbounded();
        let id = senders.insert(tx);
        BrokerStream(id, rx)
    }
}
