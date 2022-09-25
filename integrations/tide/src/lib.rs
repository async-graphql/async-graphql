//! Async-graphql integration with Tide
//!
//! # Examples
//! *[Full Example](<https://github.com/async-graphql/examples/blob/master/tide/starwars/src/main.rs>)*

#![warn(missing_docs)]
#![allow(clippy::type_complexity)]
#![allow(clippy::needless_doctest_main)]
#![forbid(unsafe_code)]

#[cfg(feature = "websocket")]
mod subscription;

use async_graphql::{
    http::MultipartOptions, ObjectType, ParseRequestError, Schema, SubscriptionType,
};
#[cfg(feature = "websocket")]
pub use subscription::GraphQLSubscription;
use tide::{
    http::{
        headers::{self, HeaderValue},
        Method,
    },
    utils::async_trait,
    Body, Request, Response, StatusCode,
};

/// Create a new GraphQL endpoint with the schema.
///
/// Default multipart options are used and batch operations are supported.
pub fn graphql<Query, Mutation, Subscription>(
    schema: Schema<Query, Mutation, Subscription>,
) -> GraphQLEndpoint<Query, Mutation, Subscription> {
    GraphQLEndpoint {
        schema,
        opts: MultipartOptions::default(),
        batch: true,
    }
}

/// A GraphQL endpoint.
///
/// This is created with the [`endpoint`](fn.endpoint.html) function.
#[non_exhaustive]
pub struct GraphQLEndpoint<Query, Mutation, Subscription> {
    /// The schema of the endpoint.
    pub schema: Schema<Query, Mutation, Subscription>,
    /// The multipart options of the endpoint.
    pub opts: MultipartOptions,
    /// Whether to support batch requests in the endpoint.
    pub batch: bool,
}

impl<Query, Mutation, Subscription> GraphQLEndpoint<Query, Mutation, Subscription> {
    /// Set the multipart options of the endpoint.
    #[must_use]
    pub fn multipart_opts(self, opts: MultipartOptions) -> Self {
        Self { opts, ..self }
    }
    /// Set whether batch requests are supported in the endpoint.
    #[must_use]
    pub fn batch(self, batch: bool) -> Self {
        Self { batch, ..self }
    }
}

// Manual impl to remove bounds on generics
impl<Query, Mutation, Subscription> Clone for GraphQLEndpoint<Query, Mutation, Subscription> {
    fn clone(&self) -> Self {
        Self {
            schema: self.schema.clone(),
            opts: self.opts,
            batch: self.batch,
        }
    }
}

#[async_trait]
impl<Query, Mutation, Subscription, TideState> tide::Endpoint<TideState>
    for GraphQLEndpoint<Query, Mutation, Subscription>
where
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
    TideState: Clone + Send + Sync + 'static,
{
    async fn call(&self, request: Request<TideState>) -> tide::Result {
        respond(
            self.schema
                .execute_batch(if self.batch {
                    receive_batch_request_opts(request, self.opts).await
                } else {
                    receive_request_opts(request, self.opts)
                        .await
                        .map(Into::into)
                }?)
                .await,
        )
    }
}

/// Convert a Tide request to a GraphQL request.
pub async fn receive_request<State: Clone + Send + Sync + 'static>(
    request: Request<State>,
) -> tide::Result<async_graphql::Request> {
    receive_request_opts(request, Default::default()).await
}

/// Convert a Tide request to a GraphQL request with options on how to receive
/// multipart.
pub async fn receive_request_opts<State: Clone + Send + Sync + 'static>(
    request: Request<State>,
    opts: MultipartOptions,
) -> tide::Result<async_graphql::Request> {
    receive_batch_request_opts(request, opts)
        .await?
        .into_single()
        .map_err(|e| tide::Error::new(StatusCode::BadRequest, e))
}

/// Convert a Tide request to a GraphQL batch request.
pub async fn receive_batch_request<State: Clone + Send + Sync + 'static>(
    request: Request<State>,
) -> tide::Result<async_graphql::BatchRequest> {
    receive_batch_request_opts(request, Default::default()).await
}

/// Convert a Tide request to a GraphQL batch request with options on how to
/// receive multipart.
pub async fn receive_batch_request_opts<State: Clone + Send + Sync + 'static>(
    mut request: Request<State>,
    opts: MultipartOptions,
) -> tide::Result<async_graphql::BatchRequest> {
    if request.method() == Method::Get {
        async_graphql::http::parse_query_string(request.url().query().unwrap_or_default())
            .map(Into::into)
            .map_err(|err| tide::Error::new(StatusCode::BadRequest, err))
    } else if request.method() == Method::Post {
        let body = request.take_body();
        let content_type = request
            .header(headers::CONTENT_TYPE)
            .and_then(|values| values.get(0))
            .map(HeaderValue::as_str);

        async_graphql::http::receive_batch_body(content_type, body, opts)
            .await
            .map_err(|e| {
                tide::Error::new(
                    match &e {
                        ParseRequestError::PayloadTooLarge => StatusCode::PayloadTooLarge,
                        _ => StatusCode::BadRequest,
                    },
                    e,
                )
            })
    } else {
        Err(tide::Error::from_str(
            StatusCode::MethodNotAllowed,
            "GraphQL only supports GET and POST requests",
        ))
    }
}

/// Convert a GraphQL response to a Tide response.
pub fn respond(resp: impl Into<async_graphql::BatchResponse>) -> tide::Result {
    let resp = resp.into();

    let mut response = Response::new(StatusCode::Ok);
    if resp.is_ok() {
        if let Some(cache_control) = resp.cache_control().value() {
            response.insert_header(headers::CACHE_CONTROL, cache_control);
        }
    }

    for (name, value) in resp.http_headers_iter() {
        if let Ok(value) = value.to_str() {
            response.append_header(name.as_str(), value);
        }
    }

    response.set_body(Body::from_json(&resp)?);
    Ok(response)
}
