use crate::BadRequest;
use async_graphql::http::MultipartOptions;
use async_graphql::{ObjectType, Schema, SubscriptionType};
use futures::TryStreamExt;
use std::io;
use std::io::ErrorKind;
use std::sync::Arc;
use warp::http::Method;
use warp::reply::Response as WarpResponse;
use warp::{Buf, Filter, Rejection, Reply};

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
///     #[field]
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
///     let filter = async_graphql_warp::graphql(schema).
///             and_then(|(schema, request): (MySchema, async_graphql::Request)| async move {
///         Ok::<_, Infallible>(async_graphql_warp::Response::from(schema.execute(request).await))
///     });
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
    let opts = Arc::new(opts);
    warp::any()
        .and(warp::method())
        .and(warp::query::raw().or(warp::any().map(String::new)).unify())
        .and(warp::header::optional::<String>("content-type"))
        .and(warp::body::stream())
        .and(warp::any().map(move || opts.clone()))
        .and(warp::any().map(move || schema.clone()))
        .and_then(
            |method,
             query: String,
             content_type,
             body,
             opts: Arc<MultipartOptions>,
             schema| async move {
                if method == Method::GET {
                    let request: async_graphql::Request = serde_urlencoded::from_str(&query)
                        .map_err(|err| warp::reject::custom(BadRequest(err.into())))?;
                    Ok::<_, Rejection>((schema, request))
                } else {
                    let request = async_graphql::http::receive_body(
                        content_type,
                        futures::TryStreamExt::map_err(body, |err| io::Error::new(ErrorKind::Other, err))
                            .map_ok(|mut buf| Buf::to_bytes(&mut buf))
                            .into_async_read(),
                        MultipartOptions::clone(&opts),
                    )
                        .await
                        .map_err(|err| warp::reject::custom(BadRequest(err.into())))?;
                    Ok::<_, Rejection>((schema, request))
                }
            },
        )
}

/// Reply for `async_graphql::Request`.
pub struct Response(async_graphql::Response);

impl From<async_graphql::Response> for Response {
    fn from(resp: async_graphql::Response) -> Self {
        Response(resp)
    }
}

fn add_cache_control(http_resp: &mut WarpResponse, resp: &async_graphql::Response) {
    if resp.is_ok() {
        if let Some(cache_control) = resp.cache_control.value() {
            if let Ok(value) = cache_control.parse() {
                http_resp.headers_mut().insert("cache-control", value);
            }
        }
    }
}

impl Reply for Response {
    fn into_response(self) -> WarpResponse {
        let mut resp = warp::reply::with_header(
            warp::reply::json(&self.0),
            "content-type",
            "application/json",
        )
        .into_response();
        add_cache_control(&mut resp, &self.0);
        resp
    }
}
