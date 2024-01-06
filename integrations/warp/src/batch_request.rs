use std::{io, io::ErrorKind, str::FromStr};

use async_graphql::{http::MultipartOptions, BatchRequest, Executor};
use futures_util::TryStreamExt;
use http::{HeaderName, HeaderValue};
use warp::{reply::Response as WarpResponse, Buf, Filter, Rejection, Reply};

use crate::GraphQLBadRequest;

/// GraphQL batch request filter
///
/// It outputs a tuple containing the `async_graphql::Executor` and
/// `async_graphql::BatchRequest`.
pub fn graphql_batch<E>(
    executor: E,
) -> impl Filter<Extract = ((E, BatchRequest),), Error = Rejection> + Clone
where
    E: Executor,
{
    graphql_batch_opts(executor, Default::default())
}

/// Similar to graphql_batch, but you can set the options with
/// :`async_graphql::MultipartOptions`.
pub fn graphql_batch_opts<E>(
    executor: E,
    opts: MultipartOptions,
) -> impl Filter<Extract = ((E, BatchRequest),), Error = Rejection> + Clone
where
    E: Executor,
{
    warp::any()
        .and(warp::get().and(warp::filters::query::raw()).and_then(
            |query_string: String| async move {
                async_graphql::http::parse_query_string(&query_string)
                    .map(Into::into)
                    .map_err(|e| warp::reject::custom(GraphQLBadRequest(e)))
            },
        ))
        .or(warp::post()
            .and(warp::header::optional::<String>("content-type"))
            .and(warp::body::stream())
            .and_then(move |content_type, body| async move {
                async_graphql::http::receive_batch_body(
                    content_type,
                    TryStreamExt::map_err(body, |e| io::Error::new(ErrorKind::Other, e))
                        .map_ok(|mut buf| {
                            let remaining = Buf::remaining(&buf);
                            Buf::copy_to_bytes(&mut buf, remaining)
                        })
                        .into_async_read(),
                    opts,
                )
                .await
                .map_err(|e| warp::reject::custom(GraphQLBadRequest(e)))
            }))
        .unify()
        .map(move |res| (executor.clone(), res))
}

/// Reply for `async_graphql::BatchRequest`.
#[derive(Debug)]
pub struct GraphQLBatchResponse(pub async_graphql::BatchResponse);

impl From<async_graphql::BatchResponse> for GraphQLBatchResponse {
    fn from(resp: async_graphql::BatchResponse) -> Self {
        GraphQLBatchResponse(resp)
    }
}

impl Reply for GraphQLBatchResponse {
    fn into_response(self) -> WarpResponse {
        let mut resp = warp::reply::with_header(
            warp::reply::json(&self.0),
            "content-type",
            "application/json",
        )
        .into_response();

        if self.0.is_ok() {
            if let Some(cache_control) = self.0.cache_control().value() {
                if let Ok(value) = cache_control.try_into() {
                    resp.headers_mut().insert("cache-control", value);
                }
            }
        }

        resp.headers_mut()
            .extend(self.0.http_headers().iter().filter_map(|(name, value)| {
                HeaderName::from_str(name.as_str())
                    .ok()
                    .zip(HeaderValue::from_bytes(value.as_bytes()).ok())
            }));
        resp
    }
}
