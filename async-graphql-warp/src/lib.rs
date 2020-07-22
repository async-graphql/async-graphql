//! Async-graphql integration with Wrap

#![warn(missing_docs)]
#![allow(clippy::type_complexity)]
#![allow(clippy::needless_doctest_main)]
#![forbid(unsafe_code)]

use async_graphql::http::{GQLRequest, StreamBody};
use async_graphql::{
    QueryDefinition, BatchQueryResponse, Data, FieldResult, IntoQueryDefinition,
    IntoQueryDefinitionOpts, ObjectType, Schema, SubscriptionType, WebSocketTransport,
};
use bytes::Bytes;
use futures::{select, SinkExt, StreamExt};
use hyper::Method;
use std::sync::Arc;
use warp::filters::ws::Message;
use warp::filters::BoxedFilter;
use warp::reject::Reject;
use warp::reply::Response;
use warp::{Filter, Rejection, Reply};

/// Bad request error
///
/// It's a wrapper of `async_graphql::ParseRequestError`.
pub struct BadRequest(pub anyhow::Error);

impl std::fmt::Debug for BadRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Reject for BadRequest {}

/// GraphQL request filter
///
/// It outputs a tuple containing the `Schema` and `QuertBuilder`.
///
/// # Examples
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
/// #[tokio::main]
/// async fn main() {
///     let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
///     let filter = async_graphql_warp::graphql(schema).and_then(|(schema, query_definition): (_, QueryDefinition)| async move {
///         Ok::<_, Infallible>(BatchGQLResponse::from(query_definition.execute(&schema).await))
///     });
///     warp::serve(filter).run(([0, 0, 0, 0], 8000)).await;
/// }
/// ```
pub fn graphql<Query, Mutation, Subscription>(
    schema: Schema<Query, Mutation, Subscription>,
) -> BoxedFilter<((Schema<Query, Mutation, Subscription>, QueryDefinition),)>
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    graphql_opts(schema, Default::default())
}

/// Similar to graphql, but you can set the options `IntoQueryBuilderOpts`.
pub fn graphql_opts<Query, Mutation, Subscription>(
    schema: Schema<Query, Mutation, Subscription>,
    opts: IntoQueryDefinitionOpts,
) -> BoxedFilter<((Schema<Query, Mutation, Subscription>, QueryDefinition),)>
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
             opts: Arc<IntoQueryDefinitionOpts>,
             schema| async move {
                if method == Method::GET {
                    let gql_request: GQLRequest =
                        serde_urlencoded::from_str(&query)
                            .map_err(|err| warp::reject::custom(BadRequest(err.into())))?;
                    let definition = gql_request
                        .into_batch_query_definition_opts(&opts)
                        .await
                        .map_err(|err| warp::reject::custom(BadRequest(err.into())))?;
                    Ok::<_, Rejection>((schema, definition))
                } else {
                    let definition = (content_type, StreamBody::new(body))
                        .into_batch_query_definition_opts(&opts)
                        .await
                        .map_err(|err| warp::reject::custom(BadRequest(err.into())))?;
                    Ok::<_, Rejection>((schema, definition))
                }
            },
        )
        .boxed()
}

/// GraphQL subscription filter
///
/// # Examples
///
/// ```no_run
/// use async_graphql::*;
/// use async_graphql_warp::*;
/// use warp::Filter;
/// use futures::{Stream, StreamExt};
/// use std::time::Duration;
///
/// struct QueryRoot;
///
/// #[Object]
/// impl QueryRoot {}
///
/// struct SubscriptionRoot;
///
/// #[Subscription]
/// impl SubscriptionRoot {
///     #[field]
///     async fn tick(&self) -> impl Stream<Item = String> {
///         tokio::time::interval(Duration::from_secs(1)).map(|n| format!("{}", n.elapsed().as_secs_f32()))
///     }
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let schema = Schema::new(QueryRoot, EmptyMutation, SubscriptionRoot);
///     let filter = async_graphql_warp::graphql_subscription(schema);
///     warp::serve(filter).run(([0, 0, 0, 0], 8000)).await;
/// }
/// ```
pub fn graphql_subscription<Query, Mutation, Subscription>(
    schema: Schema<Query, Mutation, Subscription>,
) -> BoxedFilter<(impl Reply,)>
where
    Query: ObjectType + Sync + Send + 'static,
    Mutation: ObjectType + Sync + Send + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    warp::any()
        .and(warp::ws())
        .and(warp::any().map(move || schema.clone()))
        .map(
            |ws: warp::ws::Ws, schema: Schema<Query, Mutation, Subscription>| {
                ws.on_upgrade(move |websocket| {
                    let (mut tx, rx) = websocket.split();
                    let (mut stx, srx) =
                        schema.subscription_connection(WebSocketTransport::default());

                    let mut rx = rx.fuse();
                    let mut srx = srx.fuse();

                    async move {
                        loop {
                            select! {
                                bytes = srx.next() => {
                                    if let Some(bytes) = bytes {
                                        if let Ok(text) = String::from_utf8(bytes.to_vec()) {
                                            if tx.send(Message::text(text)).await.is_err()
                                            {
                                                return;
                                            }
                                        }
                                    } else {
                                        return;
                                    }
                                }
                                msg = rx.next() => {
                                    if let Some(Ok(msg)) = msg {
                                        if msg.is_text() {
                                            if stx.send(Bytes::copy_from_slice(msg.as_bytes())).await.is_err() {
                                                return;
                                            }
                                        }
                                    } else {
                                        return;
                                    }
                                }
                            }
                        }
                    }
                })
            },
        ).map(|reply| {
        warp::reply::with_header(reply, "Sec-WebSocket-Protocol", "graphql-ws")
    })
        .boxed()
}

/// GraphQL subscription filter
///
/// Specifies that a function converts the init payload to data.
pub fn graphql_subscription_with_data<Query, Mutation, Subscription, F>(
    schema: Schema<Query, Mutation, Subscription>,
    init_context_data: F,
) -> BoxedFilter<(impl Reply,)>
where
    Query: ObjectType + Sync + Send + 'static,
    Mutation: ObjectType + Sync + Send + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
    F: Fn(serde_json::Value) -> FieldResult<Data> + Send + Sync + Clone + 'static,
{
    warp::any()
        .and(warp::ws())
        .and(warp::any().map(move || schema.clone()))
        .and(warp::any().map(move || init_context_data.clone()))
        .map(
            |ws: warp::ws::Ws, schema: Schema<Query, Mutation, Subscription>, init_context_data: F| {
                ws.on_upgrade(move |websocket| {
                    let (mut tx, rx) = websocket.split();
                    let (mut stx, srx) =
                        schema.subscription_connection(WebSocketTransport::new(init_context_data));

                    let mut rx = rx.fuse();
                    let mut srx = srx.fuse();

                    async move {
                        loop {
                            select! {
                                bytes = srx.next() => {
                                    if let Some(bytes) = bytes {
                                        if let Ok(text) = String::from_utf8(bytes.to_vec()) {
                                            if tx.send(Message::text(text)).await.is_err() {
                                                return;
                                            }
                                        }
                                    } else {
                                        return;
                                    }
                                }
                                msg = rx.next() => {
                                    if let Some(Ok(msg)) = msg {
                                        if msg.is_text() {
                                            if stx.send(Bytes::copy_from_slice(msg.as_bytes())).await.is_err() {
                                                return;
                                            }
                                        }
                                    } else {
                                        return;
                                    }
                                }
                            }
                        }
                    }
                })
            },
        ).map(|reply| {
        warp::reply::with_header(reply, "Sec-WebSocket-Protocol", "graphql-ws")
    })
        .boxed()
}

/// Batch GraphQL reply
pub struct BatchGQLResponse(async_graphql::BatchQueryResponse);

impl From<async_graphql::BatchQueryResponse> for BatchGQLResponse {
    fn from(resp: async_graphql::BatchQueryResponse) -> Self {
        BatchGQLResponse(resp)
    }
}

fn get_cache_control_batch(resp: &BatchQueryResponse) -> Option<String> {
    resp.cache_control()
        .map(|cache_control| cache_control.value())
        .flatten()
}

impl Reply for BatchGQLResponse {
    fn into_response(self) -> Response {
        let cache_control = get_cache_control_batch(&self.0);
        let gql_resp = async_graphql::http::BatchGQLResponse::from(self.0);
        let mut resp = warp::reply::with_header(
            warp::reply::json(&gql_resp),
            "content-type",
            "application/json",
        )
        .into_response();
        if let Some(cache_control_val) = cache_control {
            if let Ok(value) = cache_control_val.parse() {
                resp.headers_mut().insert("cache-control", value);
            }
        }
        resp
    }
}
