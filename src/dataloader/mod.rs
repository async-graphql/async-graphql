//! Batch loading support, used to solve N+1 problem.
//!
//! # Examples
//!
//! ```rust
//! use async_graphql::*;
//! use async_graphql::dataloader::*;
//! use std::collections::{HashSet, HashMap};
//! use std::convert::Infallible;
//! use async_graphql::dataloader::Loader;
//!
//! /// This loader simply converts the integer key into a string value.
//! struct MyLoader;
//!
//! #[async_trait::async_trait]
//! impl Loader for MyLoader {
//!     type Key = i32;
//!     type Value = String;
//!     type Error = Infallible;
//!
//!     async fn load(&self, keys: HashSet<Self::Key>) -> Result<HashMap<Self::Key, Self::Value>, Self::Error> {
//!         // Use `MyLoader` to load data.
//!         Ok(keys.iter().copied().map(|n| (n, n.to_string())).collect())
//!     }
//! }
//!
//! struct Query;
//!
//! #[Object]
//! impl Query {
//!     async fn value(&self, ctx: &Context<'_>, n: i32) -> Option<String> {
//!         ctx.data_unchecked::<DataLoader<MyLoader>>().load_one(n).await.unwrap()
//!     }
//! }
//!
//! async_std::task::block_on(async move {
//!     let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
//!         .data(DataLoader::new(MyLoader))
//!         .finish();
//!
//!     let res = schema.execute(r#"
//!         {
//!             v1: value(n: 1)
//!             v2: value(n: 2)
//!             v3: value(n: 3)
//!             v4: value(n: 4)
//!             v5: value(n: 5)
//!         }
//!     "#).await.into_result().unwrap().data;
//!
//!     assert_eq!(res, value!({
//!         "v1": "1",
//!         "v2": "2",
//!         "v3": "3",
//!         "v4": "4",
//!         "v5": "5",
//!     }));
//! });
//!
//! ```

mod cache;

use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::time::Duration;

use futures_channel::oneshot;
use futures_timer::Delay;
use futures_util::lock::Mutex;

pub use cache::{CacheStorage, CachedLoader, LruCache};

type ResSender<T> = oneshot::Sender<
    Result<HashMap<<T as Loader>::Key, <T as Loader>::Value>, <T as Loader>::Error>,
>;

struct Requests<T: Loader> {
    keys: HashSet<T::Key>,
    pending: Vec<(HashSet<T::Key>, ResSender<T>)>,
}

impl<T: Loader> Default for Requests<T> {
    fn default() -> Self {
        Self {
            keys: Default::default(),
            pending: Default::default(),
        }
    }
}

impl<T: Loader> Requests<T> {
    async fn load(self, loader: &T) {
        match loader.load(self.keys).await {
            Ok(values) => {
                for (keys, tx) in self.pending {
                    let mut res = HashMap::new();
                    for key in &keys {
                        res.extend(values.get(key).map(|value| (key.clone(), value.clone())));
                    }
                    tx.send(Ok(res)).ok();
                }
            }
            Err(err) => {
                for (_, tx) in self.pending {
                    tx.send(Err(err.clone())).ok();
                }
            }
        }
    }
}

/// Trait for batch loading.
#[async_trait::async_trait]
pub trait Loader: Send + Sync + 'static {
    /// Type of key.
    type Key: Send + Hash + Eq + Clone + 'static;

    /// type of value.
    type Value: Send + Clone + 'static;

    /// Type of error.
    type Error: Send + Clone + 'static;

    /// Load the data set specified by the `keys`.
    async fn load(
        &self,
        keys: HashSet<Self::Key>,
    ) -> Result<HashMap<Self::Key, Self::Value>, Self::Error>;
}

/// Data loader.
///
/// Reference: https://github.com/facebook/dataloader
pub struct DataLoader<T: Loader> {
    requests: Mutex<Requests<T>>,
    delay: Duration,
    max_batch_size: usize,
    loader: T,
}

impl<T: Loader> DataLoader<T> {
    /// Create a DataLoader with the `Loader` trait.
    pub fn new(loader: T) -> Self {
        Self {
            requests: Default::default(),
            delay: Duration::from_millis(20),
            max_batch_size: 1000,
            loader,
        }
    }

    /// Specify the delay time for loading data, the default is `20ms`.
    pub fn delay(self, delay: Duration) -> Self {
        Self { delay, ..self }
    }

    /// pub fn Specify the max batch size for loading data, the default is `1000`.
    ///
    /// If the keys waiting to be loaded reach the threshold, they are loaded immediately.
    pub fn max_batch_size(self, max_batch_size: usize) -> Self {
        Self {
            max_batch_size,
            ..self
        }
    }

    /// Use this `DataLoader` load a data.
    pub async fn load_one(&self, key: T::Key) -> Result<Option<T::Value>, T::Error> {
        let mut values = self.load_many(std::iter::once(key.clone())).await?;
        Ok(values.remove(&key))
    }

    /// Use this `DataLoader` to load some data.
    pub async fn load_many(
        &self,
        keys: impl Iterator<Item = T::Key>,
    ) -> Result<HashMap<T::Key, T::Value>, T::Error> {
        let (start_fetch, rx) = {
            let mut requests = self.requests.lock().await;
            let prev_count = requests.keys.len();
            let keys = keys.collect::<HashSet<_>>();
            requests.keys.extend(keys.clone());
            if requests.keys.len() == prev_count {
                return Ok(Default::default());
            }
            let (tx, rx) = oneshot::channel();
            requests.pending.push((keys, tx));
            if requests.keys.len() >= self.max_batch_size {
                let r = std::mem::take(&mut *requests);
                drop(requests);
                r.load(&self.loader).await;
                (false, rx)
            } else {
                (!requests.keys.is_empty() && prev_count == 0, rx)
            }
        };

        if start_fetch {
            Delay::new(self.delay).await;
            let requests = std::mem::take(&mut *self.requests.lock().await);
            if !requests.keys.is_empty() {
                requests.load(&self.loader).await;
            }
        }

        rx.await.unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::task;
    use std::sync::Arc;

    #[async_std::test]
    async fn test_dataloader() {
        struct MyLoader;

        #[async_trait::async_trait]
        impl Loader for MyLoader {
            type Key = i32;
            type Value = i32;
            type Error = ();

            async fn load(
                &self,
                keys: HashSet<Self::Key>,
            ) -> Result<HashMap<Self::Key, Self::Value>, Self::Error> {
                assert!(keys.len() <= 10);
                Ok(keys.into_iter().map(|k| (k, k)).collect())
            }
        }

        let loader = Arc::new(DataLoader::new(MyLoader).max_batch_size(10));
        let mut handles = Vec::new();
        for i in 0..100 {
            handles.push(task::spawn({
                let loader = loader.clone();
                async move { loader.load_one(i).await }
            }));
        }
        assert_eq!(
            futures_util::future::try_join_all(handles).await.unwrap(),
            (0..100).map(Option::Some).collect::<Vec<_>>()
        );
    }
}
