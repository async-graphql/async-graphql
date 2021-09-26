use async_graphql::{BatchResponse as GraphQLBatchResponse, ObjectType, Schema, SubscriptionType};
use poem::web::Json;
use poem::{async_trait, Endpoint, FromRequest, Request, Result};

use crate::GraphQLBatchRequest;

/// A GraphQL query endpoint.
///
/// # Example
///
/// ```
/// use poem::{route, RouteMethod};
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
/// let app = route().at("/", RouteMethod::new().post(GraphQL::new(schema)));
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
    type Output = Result<Json<GraphQLBatchResponse>>;

    async fn call(&self, req: Request) -> Self::Output {
        let (req, mut body) = req.split();
        let req = GraphQLBatchRequest::from_request(&req, &mut body).await?;
        Ok(Json(self.schema.execute_batch(req.0).await))
    }
}
