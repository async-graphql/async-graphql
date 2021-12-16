use async_graphql::{ObjectType, Schema, SubscriptionType};
use poem::{async_trait, Endpoint, FromRequest, Request, Result};

use crate::{GraphQLBatchRequest, GraphQLBatchResponse};

/// A GraphQL query endpoint.
///
/// # Example
///
/// ```
/// use poem::{Route, post};
/// use async_graphql_poem::GraphQL;
/// use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};
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
pub struct GraphQL<Query, Mutation, Subscription> {
    schema: Schema<Query, Mutation, Subscription>,
}

impl<Query, Mutation, Subscription> GraphQL<Query, Mutation, Subscription> {
    /// Create a GraphQL query endpoint.
    pub fn new(schema: Schema<Query, Mutation, Subscription>) -> Self {
        Self { schema }
    }
}

#[async_trait]
impl<Query, Mutation, Subscription> Endpoint for GraphQL<Query, Mutation, Subscription>
where
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
{
    type Output = GraphQLBatchResponse;

    async fn call(&self, req: Request) -> Result<Self::Output> {
        let (req, mut body) = req.split();
        let req = GraphQLBatchRequest::from_request(&req, &mut body).await?;
        Ok(GraphQLBatchResponse(self.schema.execute_batch(req.0).await))
    }
}
