use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::Mutex;

use super::Loader;

/// Cache storage for a loader.
pub trait CacheStorage: Send + Sync + 'static {
    /// Type of key.
    type Key: Send + Hash + Eq + Clone + 'static;

    /// Type of value.
    type Value: Send + Clone + 'static;

    /// Load `value` from cache by `key`.
    fn get(&self, key: &Self::Key) -> Option<Self::Value>;

    /// Put `value` to cache by `key`.
    fn set(&self, key: Self::Key, value: Self::Value);
}

/// Loader for the [cached](trait.LoaderExt.html#method.cached) method.
pub struct CachedLoader<C, T> {
    loader: T,
    cache: C,
}

impl<C, T> CachedLoader<C, T>
where
    C: CacheStorage<Key = T::Key, Value = T::Value>,
    T: Loader,
{
    /// Create a loader that can cache data.
    pub fn new(loader: T, cache: C) -> Self {
        Self { cache, loader }
    }
}

#[async_trait::async_trait]
impl<C, T> Loader for CachedLoader<C, T>
where
    C: CacheStorage<Key = T::Key, Value = T::Value>,
    T: Loader,
{
    type Key = T::Key;
    type Value = T::Value;
    type Error = T::Error;

    async fn load(
        &self,
        mut keys: HashSet<Self::Key>,
    ) -> Result<HashMap<Self::Key, Self::Value>, Self::Error> {
        let mut res = HashMap::new();
        for key in &keys {
            if let Some(value) = self.cache.get(key) {
                res.insert(key.clone(), value);
            }
        }
        for key in res.keys() {
            keys.remove(key);
        }
        let values = self.loader.load(keys).await?;
        for (key, value) in &values {
            self.cache.set(key.clone(), value.clone());
        }
        res.extend(values);
        Ok(res)
    }
}

/// Memory-based LRU cache.
pub struct LruCache<T: CacheStorage>(Mutex<lru::LruCache<T::Key, T::Value>>);

impl<T: CacheStorage> LruCache<T> {
    /// Creates a new LRU Cache that holds at most `cap` items.
    pub fn new(cap: usize) -> Self {
        Self(Mutex::new(lru::LruCache::new(cap)))
    }
}

impl<T: CacheStorage> CacheStorage for LruCache<T> {
    type Key = T::Key;
    type Value = T::Value;

    fn get(&self, key: &Self::Key) -> Option<Self::Value> {
        self.0.lock().unwrap().get(key).cloned()
    }

    fn set(&self, key: Self::Key, value: Self::Value) {
        self.0.lock().unwrap().put(key, value);
    }
}
