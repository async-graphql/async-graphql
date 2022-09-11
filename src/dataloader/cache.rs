use std::{
    borrow::Cow,
    collections::{hash_map::RandomState, HashMap},
    hash::{BuildHasher, Hash},
    marker::PhantomData,
};

/// Convenience trait with the constraints required for a cache key.
pub trait CacheKey: Send + Sync + Hash + Eq + Clone + 'static {}

/// Convenience trait with the constraints required for a cache value.
pub trait CacheValue: Send + Sync + Clone + 'static {}

/// Convenience trait with the constraints required for a cache state.
pub trait CacheState: Send + Sync + BuildHasher + Default + 'static {}

/// Factory for creating cache storage.
pub trait CacheFactory: Send + Sync + 'static {
    /// Create a cache storage.
    ///
    /// TODO: When GAT is stable, this memory allocation can be optimized away.
    fn create<K: CacheKey, V: CacheValue>(&self) -> Box<dyn CacheStorage<Key = K, Value = V>>;
}

/// Cache storage for [DataLoader].
pub trait CacheStorage: Send + Sync + 'static {
    /// The key type of the record.
    type Key: CacheKey;

    /// The value type of the record.
    type Value: CacheValue;

    /// Returns a reference to the value of the key in the cache or None if it
    /// is not present in the cache.
    fn get(&mut self, key: &Self::Key) -> Option<&Self::Value>;

    /// Puts a key-value pair into the cache. If the key already exists in the
    /// cache, then it updates the key's value.
    fn insert(&mut self, key: Cow<'_, Self::Key>, val: Cow<'_, Self::Value>);

    /// Removes the value corresponding to the key from the cache.
    fn remove(&mut self, key: &Self::Key);

    /// Clears the cache, removing all key-value pairs.
    fn clear(&mut self);
}

/// No cache.
pub struct NoCache;

impl CacheFactory for NoCache {
    fn create<K: CacheKey, V: CacheValue>(&self) -> Box<dyn CacheStorage<Key = K, Value = V>> {
        Box::new(NoCacheImpl {
            _mark1: PhantomData,
            _mark2: PhantomData,
        })
    }
}

struct NoCacheImpl<K, V> {
    _mark1: PhantomData<K>,
    _mark2: PhantomData<V>,
}

impl<K: CacheKey, V: CacheValue> CacheStorage for NoCacheImpl<K, V> {
    type Key = K;
    type Value = V;

    #[inline]
    fn get(&mut self, _key: &K) -> Option<&V> {
        None
    }

    #[inline]
    fn insert(&mut self, _key: Cow<'_, Self::Key>, _val: Cow<'_, Self::Value>) {}

    #[inline]
    fn remove(&mut self, _key: &K) {}

    #[inline]
    fn clear(&mut self) {}
}

/// [std::collections::HashMap] cache.
pub struct HashMapCache<S = RandomState> {
    _mark: PhantomData<S>,
}

impl<S: CacheState> HashMapCache<S> {
    /// Use specified `S: BuildHasher` to create a `HashMap` cache.
    pub fn new() -> Self {
        Self { _mark: PhantomData }
    }
}

impl Default for HashMapCache<RandomState> {
    fn default() -> Self {
        Self { _mark: PhantomData }
    }
}

impl<S: CacheState> CacheFactory for HashMapCache<S> {
    fn create<K: CacheKey, V: CacheValue>(&self) -> Box<dyn CacheStorage<Key = K, Value = V>> {
        Box::new(HashMapCacheImpl::<K, V, S>(HashMap::<K, V, S>::default()))
    }
}

struct HashMapCacheImpl<K, V, S>(HashMap<K, V, S>);

impl<K: CacheKey, V: CacheValue, S: CacheState> CacheStorage for HashMapCacheImpl<K, V, S> {
    type Key = K;
    type Value = V;

    #[inline]
    fn get(&mut self, key: &Self::Key) -> Option<&Self::Value> {
        self.0.get(key)
    }

    #[inline]
    fn insert(&mut self, key: Cow<'_, Self::Key>, val: Cow<'_, Self::Value>) {
        self.0.insert(key.into_owned(), val.into_owned());
    }

    #[inline]
    fn remove(&mut self, key: &Self::Key) {
        self.0.remove(key);
    }

    #[inline]
    fn clear(&mut self) {
        self.0.clear();
    }
}

/// LRU cache.
pub struct LruCache {
    cap: usize,
}

impl LruCache {
    /// Creates a new LRU Cache that holds at most `cap` items.
    pub fn new(cap: usize) -> Self {
        Self { cap }
    }
}

impl CacheFactory for LruCache {
    fn create<K: CacheKey, V: CacheValue>(&self) -> Box<dyn CacheStorage<Key = K, Value = V>> {
        Box::new(LruCacheImpl(lru::LruCache::new(self.cap)))
    }
}

struct LruCacheImpl<K, V>(lru::LruCache<K, V>);

impl<K: CacheKey, V: CacheValue> CacheStorage for LruCacheImpl<K, V> {
    type Key = K;
    type Value = V;

    #[inline]
    fn get(&mut self, key: &Self::Key) -> Option<&Self::Value> {
        self.0.get(key)
    }

    #[inline]
    fn insert(&mut self, key: Cow<'_, Self::Key>, val: Cow<'_, Self::Value>) {
        self.0.put(key.into_owned(), val.into_owned());
    }

    #[inline]
    fn remove(&mut self, key: &Self::Key) {
        self.0.pop(key);
    }

    #[inline]
    fn clear(&mut self) {
        self.0.clear();
    }
}
