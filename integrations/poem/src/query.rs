use async_graphql::Executor;
use poem::{async_trait, Endpoint, FromRequest, Request, Result};

use crate::{GraphQLBatchRequest, GraphQLBatchResponse};

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
    /// Create a GraphQL query endpoint.
    pub fn new(executor: E) -> Self {
        Self { executor }
    }
}

#[async_trait]
impl<E> Endpoint for GraphQL<E>
where
    E: Executor,
{
    type Output = GraphQLBatchResponse;

    async fn call(&self, req: Request) -> Result<Self::Output> {
        let (req, mut body) = req.split();
        let req = GraphQLBatchRequest::from_request(&req, &mut body).await?;
        Ok(GraphQLBatchResponse(
            self.executor.execute_batch(req.0).await,
        ))
    }
}
