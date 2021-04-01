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

mod cache;

use std::any::{Any, TypeId};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::Mutex;
use std::time::Duration;

use futures_channel::oneshot;
use futures_timer::Delay;

use fnv::FnvHashMap;

pub use cache::{CacheFactory, CacheStorage, HashMapCache, LruCache, NoCache};

#[allow(clippy::type_complexity)]
struct ResSender<K: Send + Sync + Hash + Eq + Clone + 'static, T: Loader<K>> {
    use_cache_values: HashMap<K, T::Value>,
    tx: oneshot::Sender<Result<HashMap<K, T::Value>, T::Error>>,
}

struct Requests<K: Send + Sync + Hash + Eq + Clone + 'static, T: Loader<K>> {
    keys: HashSet<K>,
    pending: Vec<(HashSet<K>, ResSender<K, T>)>,
    cache_storage: Box<dyn CacheStorage<Key = K, Value = T::Value>>,
}

impl<K: Send + Sync + Hash + Eq + Clone + 'static, T: Loader<K>> Requests<K, T> {
    fn new<C: CacheFactory>(cache_factory: &C) -> Self {
        Self {
            keys: Default::default(),
            pending: Vec::new(),
            cache_storage: cache_factory.create::<K, T::Value>(),
        }
    }

    fn take(&mut self) -> (HashSet<K>, Vec<(HashSet<K>, ResSender<K, T>)>) {
        (
            std::mem::take(&mut self.keys),
            std::mem::take(&mut self.pending),
        )
    }
}

async fn do_load<K, T, F>(
    loader: &T,
    keys: HashSet<K>,
    pending: Vec<(HashSet<K>, ResSender<K, T>)>,
    f: F,
) where
    K: Send + Sync + Hash + Eq + Clone + 'static,
    T: Loader<K>,
    F: FnOnce(&HashMap<K, T::Value>),
{
    let keys = keys.into_iter().collect::<Vec<_>>();
    match loader.load(&keys).await {
        Ok(values) => {
            f(&values);
            for (keys, sender) in pending {
                let mut res = HashMap::new();
                res.extend(sender.use_cache_values);
                for key in &keys {
                    res.extend(values.get(key).map(|value| (key.clone(), value.clone())));
                }
                sender.tx.send(Ok(res)).ok();
            }
        }
        Err(err) => {
            for (_, sender) in pending {
                sender.tx.send(Err(err.clone())).ok();
            }
        }
    }
}

/// Trait for batch loading.
#[async_trait::async_trait]
pub trait Loader<K: Send + Sync + Hash + Eq + Clone + 'static>: Send + Sync + 'static {
    /// type of value.
    type Value: Send + Sync + Clone + 'static;

    /// Type of error.
    type Error: Send + Clone + 'static;

    /// Load the data set specified by the `keys`.
    async fn load(&self, keys: &[K]) -> Result<HashMap<K, Self::Value>, Self::Error>;
}

/// Data loader.
///
/// Reference: <https://github.com/facebook/dataloader>
pub struct DataLoader<T, C = NoCache> {
    requests: Mutex<FnvHashMap<TypeId, Box<dyn Any + Sync + Send>>>,
    cache_factory: C,
    delay: Duration,
    max_batch_size: usize,
    loader: T,
}

impl<T> DataLoader<T, NoCache> {
    /// Use `Loader` to create a [DataLoader] that does not cache records.
    pub fn new(loader: T) -> Self {
        Self {
            cache_factory: NoCache,
            requests: Default::default(),
            delay: Duration::from_millis(1),
            max_batch_size: 1000,
            loader,
        }
    }
}

impl<T, C: CacheFactory> DataLoader<T, C> {
    /// Use `Loader` to create a [DataLoader] with a cache factory.
    pub fn with_cache(loader: T, cache_factory: C) -> Self {
        Self {
            cache_factory,
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

    fn update_cache<K>(&self, values: &HashMap<K, T::Value>)
    where
        K: Send + Sync + Hash + Eq + Clone + 'static,
        T: Loader<K>,
    {
        let tid = TypeId::of::<K>();
        let mut requests = self.requests.lock().unwrap();
        let typed_requests = requests
            .get_mut(&tid)
            .unwrap()
            .downcast_mut::<Requests<K, T>>()
            .unwrap();
        for (key, value) in values {
            typed_requests
                .cache_storage
                .insert(Cow::Borrowed(key), Cow::Borrowed(value));
        }
    }

    async fn immediate_load<K>(&self)
    where
        K: Send + Sync + Hash + Eq + Clone + 'static,
        T: Loader<K>,
    {
        let tid = TypeId::of::<K>();
        let (keys, pending) = {
            let mut requests = self.requests.lock().unwrap();
            let typed_requests = requests
                .get_mut(&tid)
                .unwrap()
                .downcast_mut::<Requests<K, T>>()
                .unwrap();
            typed_requests.take()
        };
        if !keys.is_empty() {
            do_load(&self.loader, keys, pending, |values| {
                // Update cache
                self.update_cache(values);
            })
            .await;
        }
    }

    /// Use this `DataLoader` to load some data.
    pub async fn load_many<K, I>(&self, keys: I) -> Result<HashMap<K, T::Value>, T::Error>
    where
        K: Send + Sync + Hash + Eq + Clone + 'static,
        I: IntoIterator<Item = K>,
        T: Loader<K>,
    {
        enum Action {
            ImmediateLoad,
            StartFetch,
            Delay,
        }

        let tid = TypeId::of::<K>();

        let (action, rx) = {
            let mut requests = self.requests.lock().unwrap();
            let typed_requests = requests
                .entry(tid)
                .or_insert_with(|| Box::new(Requests::<K, T>::new(&self.cache_factory)))
                .downcast_mut::<Requests<K, T>>()
                .unwrap();
            let prev_count = typed_requests.keys.len();
            let mut keys_set = HashSet::new();
            let mut use_cache_values = HashMap::new();

            for key in keys {
                if let Some(value) = typed_requests.cache_storage.get(&key) {
                    // Already in cache
                    use_cache_values.insert(key.clone(), value.clone());
                } else {
                    keys_set.insert(key);
                }
            }

            if !use_cache_values.is_empty() && keys_set.is_empty() {
                return Ok(use_cache_values);
            } else if use_cache_values.is_empty() && keys_set.is_empty() {
                return Ok(Default::default());
            }

            typed_requests.keys.extend(keys_set.clone());
            let (tx, rx) = oneshot::channel();
            typed_requests.pending.push((
                keys_set,
                ResSender {
                    use_cache_values,
                    tx,
                },
            ));

            if typed_requests.keys.len() >= self.max_batch_size {
                (Action::ImmediateLoad, rx)
            } else {
                (
                    if !typed_requests.keys.is_empty() && prev_count == 0 {
                        Action::StartFetch
                    } else {
                        Action::Delay
                    },
                    rx,
                )
            }
        };

        match action {
            Action::ImmediateLoad => self.immediate_load::<K>().await,
            Action::StartFetch => {
                Delay::new(self.delay).await;
                self.immediate_load::<K>().await;
            }
            Action::Delay => {}
        }

        rx.await.unwrap()
    }

    /// Feed some data into the cache.
    ///
    /// **NOTE: If the cache type is [NoCache], this function will not take effect. **
    pub async fn feed_many<K, I>(&self, values: I)
    where
        K: Send + Sync + Hash + Eq + Clone + 'static,
        I: IntoIterator<Item = (K, T::Value)>,
        T: Loader<K>,
    {
        let tid = TypeId::of::<K>();
        let mut requests = self.requests.lock().unwrap();
        let typed_requests = requests
            .entry(tid)
            .or_insert_with(|| Box::new(Requests::<K, T>::new(&self.cache_factory)))
            .downcast_mut::<Requests<K, T>>()
            .unwrap();
        for (key, value) in values {
            typed_requests
                .cache_storage
                .insert(Cow::Owned(key), Cow::Owned(value));
        }
    }

    /// Feed some data into the cache.
    ///
    /// **NOTE: If the cache type is [NoCache], this function will not take effect. **
    pub async fn feed_one<K>(&self, key: K, value: T::Value)
    where
        K: Send + Sync + Hash + Eq + Clone + 'static,
        T: Loader<K>,
    {
        self.feed_many(std::iter::once((key, value))).await;
    }

    /// Clears the cache.
    ///
    /// **NOTE: If the cache type is [NoCache], this function will not take effect. **
    pub fn clear<K>(&self)
    where
        K: Send + Sync + Hash + Eq + Clone + 'static,
        T: Loader<K>,
    {
        let tid = TypeId::of::<K>();
        let mut requests = self.requests.lock().unwrap();
        let typed_requests = requests
            .entry(tid)
            .or_insert_with(|| Box::new(Requests::<K, T>::new(&self.cache_factory)))
            .downcast_mut::<Requests<K, T>>()
            .unwrap();
        typed_requests.cache_storage.clear();
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

    #[tokio::test]
    async fn test_dataloader_load_empty() {
        let loader = DataLoader::with_cache(MyLoader, HashMapCache);
        assert!(loader.load_many::<i32, _>(vec![]).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_dataloader_with_cache() {
        let loader = DataLoader::with_cache(MyLoader, HashMapCache);
        loader.feed_many(vec![(1, 10), (2, 20), (3, 30)]).await;

        // All from the cache
        assert_eq!(
            loader.load_many(vec![1, 2, 3]).await.unwrap(),
            vec![(1, 10), (2, 20), (3, 30)].into_iter().collect()
        );

        // Part from the cache
        assert_eq!(
            loader.load_many(vec![1, 5, 6]).await.unwrap(),
            vec![(1, 10), (5, 5), (6, 6)].into_iter().collect()
        );

        // All from the loader
        assert_eq!(
            loader.load_many(vec![8, 9, 10]).await.unwrap(),
            vec![(8, 8), (9, 9), (10, 10)].into_iter().collect()
        );

        // Clear cache
        loader.clear::<i32>();
        assert_eq!(
            loader.load_many(vec![1, 2, 3]).await.unwrap(),
            vec![(1, 1), (2, 2), (3, 3)].into_iter().collect()
        );
    }
}
