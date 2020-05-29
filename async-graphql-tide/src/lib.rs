//! Async-graphql integration with Tide

#![warn(missing_docs)]
#![allow(clippy::type_complexity)]
#![allow(clippy::needless_doctest_main)]
#![forbid(unsafe_code)]

use async_graphql::http::{multipart_stream, GQLRequest, GQLResponse, StreamBody};
use async_graphql::{
    IntoQueryBuilder, IntoQueryBuilderOpts, ObjectType, ParseRequestError, QueryBuilder,
    QueryResponse, Schema, StreamResponse, SubscriptionType,
};
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
    F: Fn(QueryBuilder) -> QueryBuilder + Send,
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
    F: Fn(QueryBuilder) -> QueryBuilder + Send,
{
    let query_builder = req
        .body_graphql_opts(opts)
        .await
        .status(StatusCode::BadRequest)?;
    Ok(Response::new(StatusCode::Ok)
        .body_graphql(
            query_builder_configuration(query_builder)
                .execute(&schema)
                .await,
        )
        .status(StatusCode::InternalServerError)?)
}

/// Tide request extension
///
#[async_trait]
pub trait RequestExt<State: Send + Sync + 'static>: Sized {
    /// Convert a query to `async_graphql::QueryBuilder`.
    async fn body_graphql(self) -> Result<QueryBuilder, ParseRequestError> {
        self.body_graphql_opts(Default::default()).await
    }

    /// Similar to graphql, but you can set the options `IntoQueryBuilderOpts`.
    async fn body_graphql_opts(
        self,
        opts: IntoQueryBuilderOpts,
    ) -> Result<QueryBuilder, ParseRequestError>;
}

#[async_trait]
impl<State: Send + Sync + 'static> RequestExt<State> for Request<State> {
    async fn body_graphql_opts(
        self,
        opts: IntoQueryBuilderOpts,
    ) -> Result<QueryBuilder, ParseRequestError> {
        if self.method() == Method::Get {
            match self.query::<GQLRequest>() {
                Ok(gql_request) => gql_request.into_query_builder_opts(&opts).await,
                Err(_) => Err(ParseRequestError::Io(std::io::Error::from(
                    std::io::ErrorKind::InvalidInput,
                ))),
            }
        } else {
            let content_type = self
                .header(&headers::CONTENT_TYPE)
                .and_then(|values| values.first().map(|value| value.to_string()));
            (content_type, self).into_query_builder_opts(&opts).await
        }
    }
}

/// Tide response extension
///
pub trait ResponseExt: Sized {
    /// Set body as the result of a GraphQL query.
    fn body_graphql(self, res: async_graphql::Result<QueryResponse>) -> serde_json::Result<Self>;

    /// Set body as the result of a GraphQL streaming query.
    fn body_graphql_stream(self, res: StreamResponse) -> serde_json::Result<Self>;
}

impl ResponseExt for Response {
    fn body_graphql(self, res: async_graphql::Result<QueryResponse>) -> serde_json::Result<Self> {
        add_cache_control(self, &res).body_json(&GQLResponse(res))
    }

    fn body_graphql_stream(mut self, res: StreamResponse) -> serde_json::Result<Self> {
        match res {
            StreamResponse::Single(res) => self.body_graphql(res),
            StreamResponse::Stream(stream) => {
                // Body::from_reader required Sync, however StreamResponse does not have Sync.
                // I created an issue and got a reply that this might be fixed in the future.
                // https://github.com/http-rs/http-types/pull/144
                // Now I can only use forwarding to solve the problem.
                let mut stream =
                    Box::pin(multipart_stream(stream).map(Result::Ok::<_, std::io::Error>));
                let (mut tx, rx) = mpsc::channel(0);
                async_std::task::spawn(async move {
                    while let Some(item) = stream.next().await {
                        if tx.send(item).await.is_err() {
                            return;
                        }
                    }
                });
                self.set_body(Body::from_reader(BufReader::new(StreamBody::new(rx)), None));
                Ok(self.set_header(tide::http::headers::CONTENT_TYPE, "multipart/mixed"))
            }
        }
    }
}

fn add_cache_control(http_resp: Response, resp: &async_graphql::Result<QueryResponse>) -> Response {
    if let Ok(QueryResponse { cache_control, .. }) = resp {
        if let Some(cache_control) = cache_control.value() {
            if let Ok(header) = tide::http::headers::HeaderName::from_str("cache-control") {
                return http_resp.set_header(header, cache_control);
            }
        }
    }
    http_resp
}
