use async_graphql::{http::MultipartOptions, BatchRequest, Executor, Request};
use warp::{reply::Response as WarpResponse, Filter, Rejection, Reply};

use crate::{graphql_batch_opts, GraphQLBadRequest, GraphQLBatchResponse};

/// GraphQL request filter
///
/// It outputs a tuple containing the `async_graphql::Schema` and
/// `async_graphql::Request`.
///
/// # Examples
///
/// *[Full Example](<https://github.com/async-graphql/examples/blob/master/warp/starwars/src/main.rs>)*
///
/// ```no_run
/// use std::convert::Infallible;
///
/// use async_graphql::*;
/// use async_graphql_warp::*;
/// use warp::Filter;
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
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
/// let filter = async_graphql_warp::graphql(schema).and_then(
///     |(schema, request): (MySchema, async_graphql::Request)| async move {
///         Ok::<_, Infallible>(async_graphql_warp::GraphQLResponse::from(
///             schema.execute(request).await,
///         ))
///     },
/// );
/// warp::serve(filter).run(([0, 0, 0, 0], 8000)).await;
/// # });
/// ```
pub fn graphql<E>(
    executor: E,
) -> impl Filter<Extract = ((E, async_graphql::Request),), Error = Rejection> + Clone
where
    E: Executor,
{
    graphql_opts(executor, Default::default())
}

/// Similar to graphql, but you can set the options
/// `async_graphql::MultipartOptions`.
pub fn graphql_opts<E>(
    executor: E,
    opts: MultipartOptions,
) -> impl Filter<Extract = ((E, Request),), Error = Rejection> + Clone
where
    E: Executor,
{
    graphql_batch_opts(executor, opts).and_then(|(schema, batch): (_, BatchRequest)| async move {
        <Result<_, Rejection>>::Ok((
            schema,
            batch
                .into_single()
                .map_err(|e| warp::reject::custom(GraphQLBadRequest(e)))?,
        ))
    })
}

/// Reply for `async_graphql::Request`.
#[derive(Debug)]
pub struct GraphQLResponse(pub async_graphql::Response);

impl From<async_graphql::Response> for GraphQLResponse {
    fn from(resp: async_graphql::Response) -> Self {
        GraphQLResponse(resp)
    }
}

impl Reply for GraphQLResponse {
    fn into_response(self) -> WarpResponse {
        GraphQLBatchResponse(self.0.into()).into_response()
    }
}
