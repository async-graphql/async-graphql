//! Apollo persisted queries extension.

use std::sync::Arc;

use futures_util::lock::Mutex;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{
    extensions::{Extension, ExtensionContext, ExtensionFactory, NextPrepareRequest},
    from_value, Request, ServerError, ServerResult,
};

#[derive(Deserialize)]
struct PersistedQuery {
    version: i32,
    #[serde(rename = "sha256Hash")]
    sha256_hash: String,
}

/// Cache storage for persisted queries.
#[async_trait::async_trait]
pub trait CacheStorage: Send + Sync + Clone + 'static {
    /// Load the query by `key`.
    async fn get(&self, key: String) -> Option<String>;

    /// Save the query by `key`.
    async fn set(&self, key: String, query: String);
}

/// Memory-based LRU cache.
#[derive(Clone)]
pub struct LruCacheStorage(Arc<Mutex<lru::LruCache<String, String>>>);

impl LruCacheStorage {
    /// Creates a new LRU Cache that holds at most `cap` items.
    pub fn new(cap: usize) -> Self {
        Self(Arc::new(Mutex::new(lru::LruCache::new(cap))))
    }
}

#[async_trait::async_trait]
impl CacheStorage for LruCacheStorage {
    async fn get(&self, key: String) -> Option<String> {
        let mut cache = self.0.lock().await;
        cache.get(&key).cloned()
    }

    async fn set(&self, key: String, query: String) {
        let mut cache = self.0.lock().await;
        cache.put(key, query);
    }
}

/// Apollo persisted queries extension.
///
/// [Reference](https://www.apollographql.com/docs/react/api/link/persisted-queries/)
#[cfg_attr(docsrs, doc(cfg(feature = "apollo_persisted_queries")))]
pub struct ApolloPersistedQueries<T>(T);

impl<T: CacheStorage> ApolloPersistedQueries<T> {
    /// Creates an apollo persisted queries extension.
    pub fn new(cache_storage: T) -> ApolloPersistedQueries<T> {
        Self(cache_storage)
    }
}

impl<T: CacheStorage> ExtensionFactory for ApolloPersistedQueries<T> {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(ApolloPersistedQueriesExtension {
            storage: self.0.clone(),
        })
    }
}

struct ApolloPersistedQueriesExtension<T> {
    storage: T,
}

#[async_trait::async_trait]
impl<T: CacheStorage> Extension for ApolloPersistedQueriesExtension<T> {
    async fn prepare_request(
        &self,
        ctx: &ExtensionContext<'_>,
        mut request: Request,
        next: NextPrepareRequest<'_>,
    ) -> ServerResult<Request> {
        let res = if let Some(value) = request.extensions.remove("persistedQuery") {
            let persisted_query: PersistedQuery = from_value(value).map_err(|_| {
                ServerError::new("Invalid \"PersistedQuery\" extension configuration.", None)
            })?;
            if persisted_query.version != 1 {
                return Err(ServerError::new(
                    format!("Only the \"PersistedQuery\" extension of version \"1\" is supported, and the current version is \"{}\".", persisted_query.version), None
                ));
            }

            if request.query.is_empty() {
                if let Some(query) = self.storage.get(persisted_query.sha256_hash).await {
                    Ok(Request { query, ..request })
                } else {
                    Err(ServerError::new("PersistedQueryNotFound", None))
                }
            } else {
                let sha256_hash = format!("{:x}", Sha256::digest(request.query.as_bytes()));

                if persisted_query.sha256_hash != sha256_hash {
                    Err(ServerError::new("provided sha does not match query", None))
                } else {
                    self.storage.set(sha256_hash, request.query.clone()).await;
                    Ok(request)
                }
            }
        } else {
            Ok(request)
        };
        next.run(ctx, res?).await
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test() {
        use super::*;
        use crate::*;

        struct Query;

        #[Object(internal)]
        impl Query {
            async fn value(&self) -> i32 {
                100
            }
        }

        let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
            .extension(ApolloPersistedQueries::new(LruCacheStorage::new(256)))
            .finish();

        let mut request = Request::new("{ value }");
        request.extensions.insert(
            "persistedQuery".to_string(),
            value!({
                "version": 1,
                "sha256Hash": "854174ebed716fe24fd6659c30290aecd9bc1d17dc4f47939a1848a1b8ed3c6b",
            }),
        );

        assert_eq!(
            schema.execute(request).await.into_result().unwrap().data,
            value!({
                "value": 100
            })
        );

        let mut request = Request::new("");
        request.extensions.insert(
            "persistedQuery".to_string(),
            value!({
                "version": 1,
                "sha256Hash": "854174ebed716fe24fd6659c30290aecd9bc1d17dc4f47939a1848a1b8ed3c6b",
            }),
        );

        assert_eq!(
            schema.execute(request).await.into_result().unwrap().data,
            value!({
                "value": 100
            })
        );

        let mut request = Request::new("");
        request.extensions.insert(
            "persistedQuery".to_string(),
            value!({
                "version": 1,
                "sha256Hash": "def",
            }),
        );

        assert_eq!(
            schema.execute(request).await.into_result().unwrap_err(),
            vec![ServerError::new("PersistedQueryNotFound", None)]
        );
    }
}
