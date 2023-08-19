use std::time::Duration;

use async_graphql::{
    http::{create_multipart_mixed_stream, is_accept_multipart_mixed},
    Executor,
};
use futures_util::StreamExt;
use poem::{async_trait, Body, Endpoint, FromRequest, IntoResponse, Request, Response, Result};

use crate::{GraphQLBatchRequest, GraphQLBatchResponse, GraphQLRequest};

/// A GraphQL query endpoint.
///
/// # Example
///
/// ```
/// use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};
/// use async_graphql_poem::GraphQL;
/// use poem::{post, Route};
///
/// struct Query;
///
/// #[Object]
/// impl Query {
///     async fn value(&self) -> i32 {
///         100
///     }
/// }
///
/// type MySchema = Schema<Query, EmptyMutation, EmptySubscription>;
///
/// let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
/// let app = Route::new().at("/", post(GraphQL::new(schema)));
/// ```
pub struct GraphQL<E> {
    executor: E,
}

impl<E> GraphQL<E> {
    /// Create a GraphQL endpoint.
    pub fn new(executor: E) -> Self {
        Self { executor }
    }
}

#[async_trait]
impl<E> Endpoint for GraphQL<E>
where
    E: Executor,
{
    type Output = Response;

    async fn call(&self, req: Request) -> Result<Self::Output> {
        let is_accept_multipart_mixed = req
            .header("accept")
            .map(is_accept_multipart_mixed)
            .unwrap_or_default();

        if is_accept_multipart_mixed {
            let (req, mut body) = req.split();
            let req = GraphQLRequest::from_request(&req, &mut body).await?;
            let stream = self.executor.execute_stream(req.0, None);
            Ok(Response::builder()
                .header("content-type", "multipart/mixed; boundary=graphql")
                .body(Body::from_bytes_stream(
                    create_multipart_mixed_stream(
                        stream,
                        tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(
                            Duration::from_secs(30),
                        ))
                        .map(|_| ()),
                    )
                    .map(Ok::<_, std::io::Error>),
                )))
        } else {
            let (req, mut body) = req.split();
            let req = GraphQLBatchRequest::from_request(&req, &mut body).await?;
            Ok(GraphQLBatchResponse(self.executor.execute_batch(req.0).await).into_response())
        }
    }
}
