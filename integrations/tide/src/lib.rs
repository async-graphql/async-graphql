//! Async-graphql integration with Tide

#![warn(missing_docs)]
#![allow(clippy::type_complexity)]
#![allow(clippy::needless_doctest_main)]
#![forbid(unsafe_code)]

use async_graphql::http::MultipartOptions;
use async_graphql::{resolver_utils::ObjectType, Schema, SubscriptionType};
use async_trait::async_trait;
use std::str::FromStr;
use tide::{
    http::{headers, Method},
    Body, Request, Response, StatusCode,
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
/// #[GQLObject]
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
    configuration: F,
) -> tide::Result<Response>
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
    TideState: Clone + Send + Sync + 'static,
    F: FnOnce(async_graphql::Request) -> async_graphql::Request + Send,
{
    graphql_opts(req, schema, configuration, Default::default()).await
}

/// Similar to graphql, but you can set the options `async_graphql::MultipartOptions`.
pub async fn graphql_opts<Query, Mutation, Subscription, TideState, F>(
    req: Request<TideState>,
    schema: Schema<Query, Mutation, Subscription>,
    configuration: F,
    opts: MultipartOptions,
) -> tide::Result<Response>
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
    TideState: Clone + Send + Sync + 'static,
    F: FnOnce(async_graphql::Request) -> async_graphql::Request + Send,
{
    let request = req.body_graphql_opts(opts).await?;
    Response::new(StatusCode::Ok).body_graphql(schema.execute(configuration(request)).await)
}

/// Tide request extension
///
#[async_trait]
pub trait RequestExt<State: Clone + Send + Sync + 'static>: Sized {
    /// Convert a query to `async_graphql::Request`.
    async fn body_graphql(self) -> tide::Result<async_graphql::Request> {
        self.body_graphql_opts(Default::default()).await
    }

    /// Similar to `RequestExt::body_graphql`, but you can set the options `async_graphql::MultipartOptions`.
    async fn body_graphql_opts(
        self,
        opts: MultipartOptions,
    ) -> tide::Result<async_graphql::Request>;
}

#[async_trait]
impl<State: Clone + Send + Sync + 'static> RequestExt<State> for Request<State> {
    async fn body_graphql_opts(
        self,
        opts: MultipartOptions,
    ) -> tide::Result<async_graphql::Request> {
        if self.method() == Method::Get {
            Ok(self.query::<async_graphql::Request>()?)
        } else {
            let content_type = self
                .header(&headers::CONTENT_TYPE)
                .and_then(|values| values.get(0).map(|value| value.to_string()));
            async_graphql::http::receive_body(content_type, self, opts)
                .await
                .map_err(|err| tide::Error::new(StatusCode::BadRequest, err))
        }
    }
}

/// Tide response extension
///
pub trait ResponseExt: Sized {
    /// Set body as the result of a GraphQL query.
    fn body_graphql(self, res: async_graphql::Response) -> tide::Result<Self>;
}

impl ResponseExt for Response {
    fn body_graphql(self, res: async_graphql::Response) -> tide::Result<Self> {
        let mut resp = add_cache_control(self, &res);
        resp.set_body(Body::from_json(&res)?);
        Ok(resp)
    }
}

fn add_cache_control(mut http_resp: Response, resp: &async_graphql::Response) -> Response {
    if resp.is_ok() {
        if let Some(cache_control) = resp.cache_control.value() {
            if let Ok(header) = tide::http::headers::HeaderName::from_str("cache-control") {
                http_resp.insert_header(header, cache_control);
            }
        }
    }
    http_resp
}
