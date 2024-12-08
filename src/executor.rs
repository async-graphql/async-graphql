#[cfg(not(feature = "boxed-trait"))]
use std::future::Future;
use std::sync::Arc;

use futures_util::stream::{BoxStream, FuturesOrdered, StreamExt};

use crate::{BatchRequest, BatchResponse, Data, Request, Response};

/// Represents a GraphQL executor
#[cfg_attr(feature = "boxed-trait", async_trait::async_trait)]
pub trait Executor: Unpin + Clone + Send + Sync + 'static {
    /// Execute a GraphQL query.
    #[cfg(feature = "boxed-trait")]
    async fn execute(&self, request: Request) -> Response;

    /// Execute a GraphQL query.
    #[cfg(not(feature = "boxed-trait"))]
    fn execute(&self, request: Request) -> impl Future<Output = Response> + Send;

    /// Execute a GraphQL batch query.
    #[cfg(feature = "boxed-trait")]
    async fn execute_batch(&self, batch_request: BatchRequest) -> BatchResponse {
        match batch_request {
            BatchRequest::Single(request) => BatchResponse::Single(self.execute(request).await),
            BatchRequest::Batch(requests) => BatchResponse::Batch(
                FuturesOrdered::from_iter(
                    requests.into_iter().map(|request| self.execute(request)),
                )
                .collect()
                .await,
            ),
        }
    }

    /// Execute a GraphQL batch query.
    #[cfg(not(feature = "boxed-trait"))]
    fn execute_batch(
        &self,
        batch_request: BatchRequest,
    ) -> impl Future<Output = BatchResponse> + Send {
        async {
            match batch_request {
                BatchRequest::Single(request) => BatchResponse::Single(self.execute(request).await),
                BatchRequest::Batch(requests) => BatchResponse::Batch(
                    FuturesOrdered::from_iter(
                        requests.into_iter().map(|request| self.execute(request)),
                    )
                    .collect()
                    .await,
                ),
            }
        }
    }

    /// Execute a GraphQL subscription with session data.
    fn execute_stream(
        &self,
        request: Request,
        session_data: Option<Arc<Data>>,
    ) -> BoxStream<'static, Response>;
}
