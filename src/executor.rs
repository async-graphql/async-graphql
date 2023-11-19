use std::sync::Arc;

use futures_util::stream::{BoxStream, FuturesOrdered, StreamExt};

use crate::{BatchRequest, BatchResponse, Data, Request, Response};

/// Represents a GraphQL executor
#[async_trait::async_trait]
pub trait Executor: Unpin + Clone + Send + Sync + 'static {
    /// Execute a GraphQL query.
    async fn execute(&self, request: Request) -> Response;

    /// Execute a GraphQL batch query.
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

    /// Execute a GraphQL subscription with session data.
    fn execute_stream(
        &self,
        request: Request,
        session_data: Option<Arc<Data>>,
    ) -> BoxStream<'static, Response>;
}
