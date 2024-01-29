use std::{future::Future, sync::Arc};

use futures_util::stream::{BoxStream, FuturesOrdered, StreamExt};

use crate::{BatchRequest, BatchResponse, Data, Request, Response};

/// Represents a GraphQL executor
pub trait Executor: Unpin + Clone + Send + Sync + 'static {
    /// Execute a GraphQL query.
    fn execute(&self, request: Request) -> impl Future<Output = Response> + Send;

    /// Execute a GraphQL batch query.
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
