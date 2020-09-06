//! Async-graphql integration with Wrap

#![warn(missing_docs)]
#![allow(clippy::type_complexity)]
#![allow(clippy::needless_doctest_main)]
#![forbid(unsafe_code)]

use async_graphql::http::{GQLRequest, StreamBody};
use async_graphql::{
    Data, FieldResult, IntoQueryBuilder, IntoQueryBuilderOpts, ObjectType, QueryBuilder,
    QueryResponse, Schema, SubscriptionType, WebSocketTransport,
};
use futures::select;
use futures::{SinkExt, StreamExt};
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
///     let filter = async_graphql_warp::graphql(schema).and_then(|(schema, builder): (_, QueryBuilder)| async move {
///         Ok::<_, Infallible>(GQLResponse::from(builder.execute(&schema).await))
///     });
///     warp::serve(filter).run(([0, 0, 0, 0], 8000)).await;
/// }
/// ```
pub fn graphql<Query, Mutation, Subscription>(
    schema: Schema<Query, Mutation, Subscription>,
) -> BoxedFilter<((Schema<Query, Mutation, Subscription>, QueryBuilder),)>
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
    opts: IntoQueryBuilderOpts,
) -> BoxedFilter<((Schema<Query, Mutation, Subscription>, QueryBuilder),)>
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
             opts: Arc<IntoQueryBuilderOpts>,
             schema| async move {
                if method == Method::GET {
                    let gql_request: GQLRequest =
                        serde_urlencoded::from_str(&query)
                            .map_err(|err| warp::reject::custom(BadRequest(err.into())))?;
                    let builder = gql_request
                        .into_query_builder_opts(&opts)
                        .await
                        .map_err(|err| warp::reject::custom(BadRequest(err.into())))?;
                    Ok::<_, Rejection>((schema, builder))
                } else {
                    let builder = (content_type, StreamBody::new(body))
                        .into_query_builder_opts(&opts)
                        .await
                        .map_err(|err| warp::reject::custom(BadRequest(err.into())))?;
                    Ok::<_, Rejection>((schema, builder))
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
    graphql_subscription_with_data(schema, |_| Ok(Default::default()))
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
            |ws: warp::ws::Ws,
             schema: Schema<Query, Mutation, Subscription>,
             init_context_data: F| {
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
                                        if let Ok(text) = String::from_utf8(bytes) {
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
                                            if stx.send(msg.into_bytes()).await.is_err() {
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
        )
        .map(|reply| warp::reply::with_header(reply, "Sec-WebSocket-Protocol", "graphql-ws"))
        .boxed()
}

/// GraphQL reply
pub struct GQLResponse(async_graphql::Result<QueryResponse>);

impl From<async_graphql::Result<QueryResponse>> for GQLResponse {
    fn from(resp: async_graphql::Result<QueryResponse>) -> Self {
        GQLResponse(resp)
    }
}

fn add_cache_control(http_resp: &mut Response, resp: &async_graphql::Result<QueryResponse>) {
    if let Ok(QueryResponse { cache_control, .. }) = resp {
        if let Some(cache_control) = cache_control.value() {
            if let Ok(value) = cache_control.parse() {
                http_resp.headers_mut().insert("cache-control", value);
            }
        }
    }
}

impl Reply for GQLResponse {
    fn into_response(self) -> Response {
        let gql_resp = async_graphql::http::GQLResponse(self.0);
        let mut resp = warp::reply::with_header(
            warp::reply::json(&gql_resp),
            "content-type",
            "application/json",
        )
        .into_response();
        add_cache_control(&mut resp, &gql_resp.0);
        resp
    }
}
