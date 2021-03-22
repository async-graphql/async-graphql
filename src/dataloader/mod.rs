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
//! impl Loader<i32> for MyLoader {
//!     type Value = String;
//!     type Error = Infallible;
//!
//!     async fn load(&self, keys: &[i32]) -> Result<HashMap<i32, Self::Value>, Self::Error> {
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
//! tokio::runtime::Runtime::new().unwrap().block_on(async move {
//!     let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
//!     let query = r#"
//!         {
//!             v1: value(n: 1)
//!             v2: value(n: 2)
//!             v3: value(n: 3)
//!             v4: value(n: 4)
//!             v5: value(n: 5)
//!         }
//!     "#;
//!     let request = Request::new(query).data(DataLoader::new(MyLoader));
//!     let res = schema.execute(request).await.into_result().unwrap().data;
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

use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::time::Duration;

use futures_channel::oneshot;
use futures_timer::Delay;
use futures_util::lock::Mutex;

use fnv::FnvHashMap;

#[allow(clippy::type_complexity)]
struct ResSender<K: Send + Hash + Eq + Clone + 'static, T: Loader<K>>(
    oneshot::Sender<Result<HashMap<K, T::Value>, T::Error>>,
);

struct Requests<K: Send + Hash + Eq + Clone + 'static, T: Loader<K>> {
    keys: HashSet<K>,
    pending: Vec<(HashSet<K>, ResSender<K, T>)>,
}

impl<K: Send + Hash + Eq + Clone + 'static, T: Loader<K>> Default for Requests<K, T> {
    fn default() -> Self {
        Self {
            keys: Default::default(),
            pending: Default::default(),
        }
    }
}

impl<K: Send + Hash + Eq + Clone + 'static, T: Loader<K>> Requests<K, T> {
    async fn load(self, loader: &T) {
        let keys = self.keys.into_iter().collect::<Vec<_>>();
        match loader.load(&keys).await {
            Ok(values) => {
                for (keys, tx) in self.pending {
                    let mut res = HashMap::new();
                    for key in &keys {
                        res.extend(values.get(key).map(|value| (key.clone(), value.clone())));
                    }
                    tx.0.send(Ok(res)).ok();
                }
            }
            Err(err) => {
                for (_, tx) in self.pending {
                    tx.0.send(Err(err.clone())).ok();
                }
            }
        }
    }
}

/// Trait for batch loading.
#[async_trait::async_trait]
pub trait Loader<K: Send + Hash + Eq + Clone + 'static>: Send + Sync + 'static {
    /// type of value.
    type Value: Send + Clone + 'static;

    /// Type of error.
    type Error: Send + Clone + 'static;

    /// Load the data set specified by the `keys`.
    async fn load(&self, keys: &[K]) -> Result<HashMap<K, Self::Value>, Self::Error>;
}

/// Data loader.
///
/// Reference: <https://github.com/facebook/dataloader>
pub struct DataLoader<T> {
    requests: Mutex<FnvHashMap<TypeId, Box<dyn Any + Sync + Send>>>,
    delay: Duration,
    max_batch_size: usize,
    loader: T,
}

impl<T> DataLoader<T> {
    /// Create a DataLoader with the `Loader` trait.
    pub fn new(loader: T) -> Self {
        Self {
            requests: Default::default(),
            delay: Duration::from_millis(1),
            max_batch_size: 1000,
            loader,
        }
    }

    /// Specify the delay time for loading data, the default is `1ms`.
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

    /// Get the loader.
    #[inline]
    pub fn loader(&self) -> &T {
        &self.loader
    }

    /// Use this `DataLoader` load a data.
    pub async fn load_one<K>(&self, key: K) -> Result<Option<T::Value>, T::Error>
    where
        K: Send + Sync + Hash + Eq + Clone + 'static,
        T: Loader<K>,
    {
        let mut values = self.load_many(std::iter::once(key.clone())).await?;
        Ok(values.remove(&key))
    }

    /// Use this `DataLoader` to load some data.
    pub async fn load_many<K>(
        &self,
        keys: impl Iterator<Item = K>,
    ) -> Result<HashMap<K, T::Value>, T::Error>
    where
        K: Send + Sync + Hash + Eq + Clone + 'static,
        T: Loader<K>,
    {
        let tid = TypeId::of::<K>();

        let (start_fetch, rx) = {
            let mut requests = self.requests.lock().await;
            let typed_requests = requests
                .entry(tid)
                .or_insert_with(|| Box::new(Requests::<K, T>::default()))
                .downcast_mut::<Requests<K, T>>()
                .unwrap();
            let prev_count = typed_requests.keys.len();
            let keys = keys.collect::<HashSet<_>>();
            typed_requests.keys.extend(keys.clone());
            let (tx, rx) = oneshot::channel();
            typed_requests.pending.push((keys, ResSender(tx)));
            if typed_requests.keys.len() >= self.max_batch_size {
                let r = std::mem::take(&mut *typed_requests);
                drop(requests);
                r.load(&self.loader).await;
                (false, rx)
            } else {
                (!typed_requests.keys.is_empty() && prev_count == 0, rx)
            }
        };

        if start_fetch {
            Delay::new(self.delay).await;
            let mut requests = self.requests.lock().await;
            let typed_requests = requests
                .get_mut(&tid)
                .unwrap()
                .downcast_mut::<Requests<K, T>>()
                .unwrap();
            let typed_requests = std::mem::take(typed_requests);
            drop(requests);
            if !typed_requests.keys.is_empty() {
                typed_requests.load(&self.loader).await;
            }
        }

        rx.await.unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    struct MyLoader;

    #[async_trait::async_trait]
    impl Loader<i32> for MyLoader {
        type Value = i32;
        type Error = ();

        async fn load(&self, keys: &[i32]) -> Result<HashMap<i32, Self::Value>, Self::Error> {
            assert!(keys.len() <= 10);
            Ok(keys.iter().copied().map(|k| (k, k)).collect())
        }
    }

    #[async_trait::async_trait]
    impl Loader<i64> for MyLoader {
        type Value = i64;
        type Error = ();

        async fn load(&self, keys: &[i64]) -> Result<HashMap<i64, Self::Value>, Self::Error> {
            assert!(keys.len() <= 10);
            Ok(keys.iter().copied().map(|k| (k, k)).collect())
        }
    }

    #[tokio::test]
    async fn test_dataloader() {
        let loader = Arc::new(DataLoader::new(MyLoader).max_batch_size(10));
        assert_eq!(
            futures_util::future::try_join_all((0..100i32).map({
                let loader = loader.clone();
                move |n| {
                    let loader = loader.clone();
                    async move { loader.load_one(n).await }
                }
            }))
            .await
            .unwrap(),
            (0..100).map(Option::Some).collect::<Vec<_>>()
        );

        assert_eq!(
            futures_util::future::try_join_all((0..100i64).map({
                let loader = loader.clone();
                move |n| {
                    let loader = loader.clone();
                    async move { loader.load_one(n).await }
                }
            }))
            .await
            .unwrap(),
            (0..100).map(Option::Some).collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_duplicate_keys() {
        let loader = Arc::new(DataLoader::new(MyLoader).max_batch_size(10));
        assert_eq!(
            futures_util::future::try_join_all([1, 3, 5, 1, 7, 8, 3, 7].iter().copied().map({
                let loader = loader.clone();
                move |n| {
                    let loader = loader.clone();
                    async move { loader.load_one(n).await }
                }
            }))
            .await
            .unwrap(),
            [1, 3, 5, 1, 7, 8, 3, 7]
                .iter()
                .copied()
                .map(Option::Some)
                .collect::<Vec<_>>()
        );
    }
}
