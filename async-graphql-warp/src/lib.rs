//! Async-graphql integration with Wrap

#![warn(missing_docs)]
#![allow(clippy::type_complexity)]

use async_graphql::http::StreamBody;
use async_graphql::{
    IntoQueryBuilder, ObjectType, QueryBuilder, Schema, SubscriptionType, WebSocketTransport,
};
use bytes::Bytes;
use futures::select;
use futures::{SinkExt, StreamExt};
use warp::filters::ws::Message;
use warp::filters::BoxedFilter;
use warp::reject::Reject;
use warp::{Filter, Rejection, Reply};

/// Bad request error
///
/// It's a wrapper of `async_graphql::ParseRequestError`.
pub struct BadRequest(pub async_graphql::ParseRequestError);

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
///
/// ```no_run
///
/// use async_graphql::*;
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
/// let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
/// let filter = async_graphql_warp::graphql(schema).and_then(|schema, builder| async move {
///     let resp = builder.execute(&schema).await;
///     Ok::<_, Infallible>(warp::reply::json(&GQLResponse(resp)).into_response())
/// });
/// warp::serve(filter).run(([0, 0, 0, 0], 8000)).await;
/// ```
pub fn graphql<Query, Mutation, Subscription>(
    schema: Schema<Query, Mutation, Subscription>,
) -> BoxedFilter<((Schema<Query, Mutation, Subscription>, QueryBuilder),)>
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    warp::any()
        .and(warp::post())
        .and(warp::header::optional::<String>("content-type"))
        .and(warp::body::stream())
        .and(warp::any().map(move || schema.clone()))
        .and_then(|content_type, body, schema| async move {
            let builder = (content_type, StreamBody::new(body))
                .into_query_builder()
                .await
                .map_err(|err| warp::reject::custom(BadRequest(err)))?;
            Ok::<_, Rejection>((schema, builder))
        })
        .boxed()
}

/// GraphQL subscription filter
///
/// # Examples
///
/// ```no_run
/// use async_graphql::*;
/// use warp::Filter;
/// use futures::{Stream, StreamExt};
/// use tokio::time::Duration;
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
///     async fn tick(&self) -> impl Stream<String> {
///         tokio::time::interval(Duration::from_secs(1)).map(|n| format!("{}", n.elapsed().as_secs_f32()))
///     }
/// }
///
/// let schema = Schema::new(QueryRoot, EmptyMutation, SubscriptionRoot);
/// let filter = async_graphql_warp::graphql_subscription(schema);
/// warp::serve(filter).run(([0, 0, 0, 0], 8000)).await;
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
                                        if tx
                                            .send(Message::text(unsafe {
                                                String::from_utf8_unchecked(bytes.to_vec())
                                            }))
                                            .await
                                            .is_err()
                                        {
                                            return;
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
