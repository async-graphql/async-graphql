//! Async-graphql integration with Tide

#![warn(missing_docs)]
#![allow(clippy::type_complexity)]
#![allow(clippy::needless_doctest_main)]

use async_graphql::http::{multipart_stream, GQLRequest, GQLResponse, StreamBody};
use async_graphql::{
    IntoQueryBuilder, IntoQueryBuilderOpts, ObjectType, ParseRequestError, QueryBuilder,
    QueryResponse, Schema, StreamResponse, SubscriptionType,
};
use async_trait::async_trait;
use futures::io::BufReader;
use futures::StreamExt;
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
        self.body_json(&GQLResponse(res))
    }

    fn body_graphql_stream(mut self, res: StreamResponse) -> serde_json::Result<Self> {
        match res {
            StreamResponse::Single(res) => self.body_graphql(res),
            StreamResponse::Stream(stream) => {
                let r = BufReader::new(StreamBody::new(Box::pin(
                    multipart_stream(stream).map(Result::Ok::<_, std::io::Error>),
                )));
                self.set_body(Body::from_reader(r, None));
                Ok(self.set_header(tide::http::headers::CONTENT_TYPE, "multipart/mixed"))
            }
        }
    }
}
