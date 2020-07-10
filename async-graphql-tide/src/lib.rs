//! Async-graphql integration with Tide

#![warn(missing_docs)]
#![allow(clippy::type_complexity)]
#![allow(clippy::needless_doctest_main)]
#![forbid(unsafe_code)]

use async_graphql::http::{multipart_stream, StreamBody, BatchGQLResponse, BatchGQLRequest};
use async_graphql::{IntoBatchQueryBuilder, IntoQueryBuilderOpts, ObjectType, BatchQueryBuilder, QueryResponse, Schema, StreamResponse, SubscriptionType, BatchQueryResponse, BatchStreamResponse};
use async_trait::async_trait;
use futures::channel::mpsc;
use futures::io::BufReader;
use futures::{SinkExt, StreamExt};
use std::str::FromStr;
use tide::{
    http::{headers, Method},
    Body, Request, Response, Status, StatusCode,
};

/// GraphQL request handler
///
///
/// # Examples
/// *[Full Example](<https://github.com/async-graphql/examples/blob/master/tide/starwars/src/main.rs>)*
///
/// ```no_run
/// use async_graphql::*;
/// use async_std::task;
/// use tide::Request;
///
/// struct QueryRoot;
/// #[Object]
/// impl QueryRoot {
///     #[field(desc = "Returns the sum of a and b")]
///     async fn add(&self, a: i32, b: i32) -> i32 {
///         a + b
///     }
/// }
///
/// fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
///     task::block_on(async {
///         let mut app = tide::new();
///         app.at("/").post(|req: Request<()>| async move {
///             let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();
///             async_graphql_tide::graphql(req, schema, |query_builder| query_builder).await
///         });
///         app.listen("0.0.0.0:8000").await?;
///
///         Ok(())
///     })
/// }
/// ```
pub async fn graphql<Query, Mutation, Subscription, TideState, F>(
    req: Request<TideState>,
    schema: Schema<Query, Mutation, Subscription>,
    query_builder_configuration: F,
) -> tide::Result<Response>
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
    TideState: Send + Sync + 'static,
    F: Fn(BatchQueryBuilder) -> BatchQueryBuilder + Send,
{
    graphql_opts(req, schema, query_builder_configuration, Default::default()).await
}

/// Similar to graphql, but you can set the options `IntoQueryBuilderOpts`.
pub async fn graphql_opts<Query, Mutation, Subscription, TideState, F>(
    req: Request<TideState>,
    schema: Schema<Query, Mutation, Subscription>,
    query_builder_configuration: F,
    opts: IntoQueryBuilderOpts,
) -> tide::Result<Response>
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
    TideState: Send + Sync + 'static,
    F: Fn(BatchQueryBuilder) -> BatchQueryBuilder + Send,
{
    let query_builder = req.body_graphql_opts(opts).await?;
    Response::new(StatusCode::Ok).body_graphql(
        query_builder_configuration(query_builder)
            .execute(&schema)
            .await,
    )
}

/// Tide request extension
///
#[async_trait]
pub trait RequestExt<State: Send + Sync + 'static>: Sized {
    /// Convert a query to `async_graphql::QueryBuilder`.
    async fn body_graphql(self) -> tide::Result<BatchQueryBuilder> {
        self.body_graphql_opts(Default::default()).await
    }

    /// Similar to graphql, but you can set the options `IntoQueryBuilderOpts`.
    async fn body_graphql_opts(self, opts: IntoQueryBuilderOpts) -> tide::Result<BatchQueryBuilder>;
}

#[async_trait]
impl<State: Send + Sync + 'static> RequestExt<State> for Request<State> {
    async fn body_graphql_opts(self, opts: IntoQueryBuilderOpts) -> tide::Result<BatchQueryBuilder> {
        if self.method() == Method::Get {
            let gql_request: BatchGQLRequest = self.query::<BatchGQLRequest>()?;
            let builder = gql_request
                .into_batch_query_builder_opts(&opts)
                .await
                .status(StatusCode::BadRequest)?;
            Ok(builder)
        } else {
            let content_type = self
                .header(&headers::CONTENT_TYPE)
                .and_then(|values| values.get(0).map(|value| value.to_string()));
            Ok((content_type, self).into_batch_query_builder_opts(&opts).await?)
        }
    }
}

/// Tide response extension
///
pub trait ResponseExt: Sized {
    /// Set body as the result of a GraphQL query.
    fn body_graphql(self, res: BatchQueryResponse) -> tide::Result<Self>;

    /// Set body as the result of a GraphQL streaming query.
    fn body_graphql_stream(self, res: BatchStreamResponse) -> tide::Result<Self>;
}

impl ResponseExt for Response {
    fn body_graphql(self, res: BatchQueryResponse) -> tide::Result<Self> {
        let mut resp = add_cache_control(self, &res);
        resp.set_body(Body::from_json(&BatchGQLResponse::from(res))?);
        Ok(resp)
    }

    fn body_graphql_stream(mut self, res: BatchStreamResponse) -> tide::Result<Self> {
        todo!()
        // match res {
        //     StreamResponse::Single(res) => self.body_graphql(res),
        //     StreamResponse::Stream(stream) => {
        //         // Body::from_reader required Sync, however StreamResponse does not have Sync.
        //         // I created an issue and got a reply that this might be fixed in the future.
        //         // https://github.com/http-rs/http-types/pull/144
        //         // Now I can only use forwarding to solve the problem.
        //         let mut stream =
        //             Box::pin(multipart_stream(stream).map(Result::Ok::<_, std::io::Error>));
        //         let (mut tx, rx) = mpsc::channel(0);
        //         async_std::task::spawn(async move {
        //             while let Some(item) = stream.next().await {
        //                 if tx.send(item).await.is_err() {
        //                     return;
        //                 }
        //             }
        //         });
        //         self.set_body(Body::from_reader(BufReader::new(StreamBody::new(rx)), None));
        //         self.insert_header(tide::http::headers::CONTENT_TYPE, "multipart/mixed");
        //         Ok(self)
        //     }
        // }
    }
}

fn add_cache_control(
    mut http_resp: Response,
    resp: &BatchQueryResponse,
) -> Response {
    if let Some(cache_control) = resp.cache_control() {
        if let Some(cache_control) = cache_control.value() {
            if let Ok(header) = tide::http::headers::HeaderName::from_str("cache-control") {
                http_resp.insert_header(header, cache_control);
                return http_resp;
            }
        }
    }
    http_resp
}
