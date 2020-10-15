use async_graphql::http::MultipartOptions;
use async_graphql::{BatchRequest, ObjectType, Request, Schema, SubscriptionType};
use warp::reply::Response as WarpResponse;
use warp::{Filter, Rejection, Reply};

use crate::{graphql_batch_opts, BadRequest, BatchResponse};

/// GraphQL request filter
///
/// It outputs a tuple containing the `async_graphql::Schema` and `async_graphql::Request`.
///
/// # Examples
///
/// *[Full Example](<https://github.com/async-graphql/examples/blob/master/warp/starwars/src/main.rs>)*
///
/// ```no_run
///
/// use async_graphql::*;
/// use async_graphql_warp::*;
/// use warp::Filter;
/// use std::convert::Infallible;
///
/// struct QueryRoot;
///
/// #[Object]
/// impl QueryRoot {
///     async fn value(&self, ctx: &Context<'_>) -> i32 {
///         unimplemented!()
///     }
/// }
///
/// type MySchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;
///
/// #[tokio::main]
/// async fn main() {
///     let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
///     let filter = async_graphql_warp::graphql(schema)
///         .and_then(|(schema, request): (MySchema, async_graphql::Request)| async move {
///             Ok::<_, Infallible>(async_graphql_warp::Response::from(schema.execute(request).await))
///         });
///     warp::serve(filter).run(([0, 0, 0, 0], 8000)).await;
/// }
/// ```
pub fn graphql<Query, Mutation, Subscription>(
    schema: Schema<Query, Mutation, Subscription>,
) -> impl Filter<
    Extract = ((
        Schema<Query, Mutation, Subscription>,
        async_graphql::Request,
    ),),
    Error = Rejection,
> + Clone
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    graphql_opts(schema, Default::default())
}

/// Similar to graphql, but you can set the options `async_graphql::MultipartOptions`.
pub fn graphql_opts<Query, Mutation, Subscription>(
    schema: Schema<Query, Mutation, Subscription>,
    opts: MultipartOptions,
) -> impl Filter<Extract = ((Schema<Query, Mutation, Subscription>, Request),), Error = Rejection> + Clone
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    graphql_batch_opts(schema, opts).and_then(|(schema, batch): (_, BatchRequest)| async move {
        <Result<_, Rejection>>::Ok((
            schema,
            batch
                .into_single()
                .map_err(|e| warp::reject::custom(BadRequest(e)))?,
        ))
    })
}

/// Reply for `async_graphql::Request`.
#[derive(Debug)]
pub struct Response(pub async_graphql::Response);

impl From<async_graphql::Response> for Response {
    fn from(resp: async_graphql::Response) -> Self {
        Response(resp)
    }
}

impl Reply for Response {
    fn into_response(self) -> WarpResponse {
        BatchResponse(self.0.into()).into_response()
    }
}
