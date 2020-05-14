//! Async-graphql integration with Tide

#![warn(missing_docs)]
#![allow(clippy::type_complexity)]
#![allow(clippy::needless_doctest_main)]

use async_graphql::http::GQLResponse;
use async_graphql::{
    IntoQueryBuilder, IntoQueryBuilderOpts, ObjectType, ParseRequestError, QueryBuilder, Schema,
    SubscriptionType,
};
use async_trait::async_trait;
use tide::{http::headers, Request, Response, Status, StatusCode};

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
    let query_builder = req.graphql().await.status(StatusCode::BadRequest)?;
    let resp = GQLResponse(
        query_builder_configuration(query_builder)
            .execute(&schema)
            .await,
    );
    Ok(Response::new(StatusCode::Ok).body_json(&resp)?)
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
        .graphql_opts(opts)
        .await
        .status(StatusCode::BadRequest)?;
    let resp = GQLResponse(
        query_builder_configuration(query_builder)
            .execute(&schema)
            .await,
    );
    Ok(Response::new(StatusCode::Ok).body_json(&resp)?)
}

/// Tide request extension
///
#[async_trait]
pub trait RequestExt<State: Send + Sync + 'static>: Sized {
    /// Convert a query to `async_graphql::QueryBuilder`.
    async fn graphql(self) -> Result<QueryBuilder, ParseRequestError> {
        self.graphql_opts(Default::default()).await
    }

    /// Similar to graphql, but you can set the options `IntoQueryBuilderOpts`.
    async fn graphql_opts(
        self,
        opts: IntoQueryBuilderOpts,
    ) -> Result<QueryBuilder, ParseRequestError>;
}

#[async_trait]
impl<State: Send + Sync + 'static> RequestExt<State> for Request<State> {
    async fn graphql_opts(
        self,
        opts: IntoQueryBuilderOpts,
    ) -> Result<QueryBuilder, ParseRequestError> {
        let content_type = self
            .header(&headers::CONTENT_TYPE)
            .and_then(|values| values.first().map(|value| value.to_string()));
        (content_type, self).into_query_builder_opts(&opts).await
    }
}
