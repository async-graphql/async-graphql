use futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use futures::task::{Context, Poll};
use futures::{Stream, StreamExt};
use once_cell::sync::Lazy;
use std::marker::PhantomData;
use slab::Slab;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Mutex;

static SUBSCRIBERS: Lazy<Mutex<HashMap<TypeId, Box<dyn Any + Send>>>> = Lazy::new(Default::default);

struct Senders<T>(Slab<UnboundedSender<T>>);

struct BrokerStream<T: Sync + Send + Clone + 'static>(usize, UnboundedReceiver<T>);

fn with_senders<T, F, R>(f: F) -> R
where
    T: Sync + Send + Clone + 'static,
    F: FnOnce(&mut Senders<T>) -> R,
{
    let mut map = SUBSCRIBERS.lock().unwrap();
    let senders = map
        .entry(TypeId::of::<Senders<T>>())
        .or_insert_with(|| Box::new(Senders::<T>(Default::default())));
    f(senders.downcast_mut::<Senders<T>>().unwrap())
}

impl<T: Sync + Send + Clone + 'static> Drop for BrokerStream<T> {
    fn drop(&mut self) {
        with_senders::<T, _, _>(|senders| senders.0.remove(self.0));
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
    /// Publish a message that all subscription streams can receive.
    pub fn publish(msg: T) {
        with_senders::<T, _, _>(|senders| {
            for (_, sender) in senders.0.iter_mut() {
                sender.start_send(msg.clone()).ok();
            }
        });
    }

    /// Subscribe to the message of the specified type and returns a `Stream`.
    pub fn subscribe() -> impl Stream<Item = T> {
        with_senders::<T, _, _>(|senders| {
            let (tx, rx) = mpsc::unbounded();
            let id = senders.0.insert(tx);
            BrokerStream(id, rx)
        })
    }
}
