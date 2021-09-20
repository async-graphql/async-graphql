//! Relay persisted queries extension.
//!
//! The relay compiler can compile queries to store them into files or into databases. It's useful
//! because:
//!
//! * Then client can just send an md5 hash, which is shorter.
//! * The server can now allowlist queries which improves security
//!
//! This extension will allow you to implement the relay persisted queries within `async-graphql`
//! by providing the Get method to access to the queries given a md5 hash and allow you to
//! implement an allowlist for your queries.
//!
//! # References
//!
//! * [Relay documentation about persisted queries](https://relay.dev/docs/guides/persisted-queries/)
use std::sync::Arc;

use crate::extensions::{Extension, ExtensionContext, ExtensionFactory, NextPrepareRequest};
use crate::{Request, ServerError, ServerResult};

/// Relay persisted queries extension.
///
/// [Reference](https://relay.dev/docs/guides/persisted-queries/)
#[cfg_attr(docsrs, doc(cfg(feature = "relay_persisted_queries")))]
pub struct RelayPersistedQueries<T>(T);

/// Cache storage for persisted queries.
/// The Cache storage describe a way to get a query
#[async_trait::async_trait]
pub trait RelayCacheStorage: Send + Sync + Clone + 'static {
    /// Load the query by `key`.
    async fn get<S: AsRef<str>>(&self, key: S) -> Option<String>;
}

impl<T: RelayCacheStorage> RelayPersistedQueries<T> {
    /// Creates a relay persisted queries extension.
    pub fn new(cache_storage: T) -> RelayPersistedQueries<T> {
        Self(cache_storage)
    }
}

impl<T: RelayCacheStorage> ExtensionFactory for RelayPersistedQueries<T> {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(RelayPersistedQueriesExtension {
            storage: self.0.clone(),
        })
    }
}

struct RelayPersistedQueriesExtension<T> {
    storage: T,
}

#[async_trait::async_trait]
impl<T: RelayCacheStorage> Extension for RelayPersistedQueriesExtension<T> {
    async fn prepare_request(
        &self,
        ctx: &ExtensionContext<'_>,
        request: Request,
        next: NextPrepareRequest<'_>,
    ) -> ServerResult<Request> {
        let res: Result<Request, ServerError> = if let Some(doc_id) = &request.doc_id {
            let persisted_query = self.storage.get(doc_id).await.ok_or(ServerError::new("Run mode is not available yet. You cannot use a doc_id not registred by the server.", None))?;

            Ok(Request {
                query: persisted_query,
                ..request
            })
        } else {
            Ok(request)
        };
        next.run(ctx, res?).await
    }
}
