use crate::BadRequest;
use async_graphql::http::MultipartOptions;
use async_graphql::{ObjectType, Schema, SubscriptionType};
use futures::TryStreamExt;
use std::io;
use std::io::ErrorKind;
use std::sync::Arc;
use warp::reply::Response as WarpResponse;
use warp::{Buf, Filter, Rejection, Reply};

/// GraphQL batch request filter
///
/// It outputs a tuple containing the `async_graphql::Schema` and `async_graphql::BatchRequest`.
pub fn graphql_batch<Query, Mutation, Subscription>(
    schema: Schema<Query, Mutation, Subscription>,
) -> impl Filter<
    Extract = ((
        Schema<Query, Mutation, Subscription>,
        async_graphql::BatchRequest,
    ),),
    Error = Rejection,
> + Clone
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    graphql_batch_opts(schema, Default::default())
}

/// Similar to graphql_batch, but you can set the options `async_graphql::MultipartOptions`.
pub fn graphql_batch_opts<Query, Mutation, Subscription>(
    schema: Schema<Query, Mutation, Subscription>,
    opts: MultipartOptions,
) -> impl Filter<
    Extract = ((
        Schema<Query, Mutation, Subscription>,
        async_graphql::BatchRequest,
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
        .and(warp::header::optional::<String>("content-type"))
        .and(warp::body::stream())
        .and(warp::any().map(move || opts.clone()))
        .and(warp::any().map(move || schema.clone()))
        .and_then(
            |content_type, body, opts: Arc<MultipartOptions>, schema| async move {
                let request = async_graphql::http::receive_batch_body(
                    content_type,
                    futures::TryStreamExt::map_err(body, |err| {
                        io::Error::new(ErrorKind::Other, err)
                    })
                    .map_ok(|mut buf| Buf::to_bytes(&mut buf))
                    .into_async_read(),
                    MultipartOptions::clone(&opts),
                )
                .await
                .map_err(|err| warp::reject::custom(BadRequest(err.into())))?;
                Ok::<_, Rejection>((schema, request))
            },
        )
}

/// Reply for `async_graphql::BatchRequest`.
pub struct BatchResponse(async_graphql::BatchResponse);

impl From<async_graphql::BatchResponse> for BatchResponse {
    fn from(resp: async_graphql::BatchResponse) -> Self {
        BatchResponse(resp)
    }
}

fn add_cache_control(http_resp: &mut WarpResponse, resp: &async_graphql::BatchResponse) {
    if resp.is_ok() {
        if let Some(cache_control) = resp.cache_control().value() {
            if let Ok(value) = cache_control.parse() {
                http_resp.headers_mut().insert("cache-control", value);
            }
        }
    }
}

impl Reply for BatchResponse {
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
