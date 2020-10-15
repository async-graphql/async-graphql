//! Async-graphql integration with Rocket.
//!
//! Note: This integrates with the unreleased version 0.5 of Rocket, and so breaking changes in
//! both this library and Rocket are to be expected.
//!
//! To configure options for sending and receiving multipart requests, add your instance of
//! `MultipartOptions` to the state managed by Rocket (`.manage(your_multipart_options)`).
//!
//! **[Full Example](<https://github.com/async-graphql/examples/blob/master/rocket/starwars/src/main.rs>)**

#![warn(missing_docs)]
#![forbid(unsafe_code)]

use std::io::Cursor;

use async_graphql::http::MultipartOptions;
use async_graphql::{ObjectType, ParseRequestError, Schema, SubscriptionType};
use rocket::{
    data::{self, Data, FromData, ToByteUnit},
    http::{ContentType, Header, Status},
    request::{self, FromQuery},
    response::{self, Responder},
};
use serde::de::Deserialize;
use tokio_util::compat::Tokio02AsyncReadCompatExt;

use query_deserializer::QueryDeserializer;

mod query_deserializer;

/// A batch request which can be extracted from a request's body.
///
/// # Examples
///
/// ```ignore
/// #[rocket::post("/graphql", data = "<request>", format = "application/json")]
/// async fn graphql_request(schema: State<'_, ExampleSchema>, request: BatchRequest) -> Response {
///     request.execute(&schema).await
/// }
/// ```
#[derive(Debug)]
pub struct BatchRequest(pub async_graphql::BatchRequest);

impl BatchRequest {
    /// Shortcut method to execute the request on the schema.
    pub async fn execute<Query, Mutation, Subscription>(
        self,
        schema: &Schema<Query, Mutation, Subscription>,
    ) -> Response
    where
        Query: ObjectType + Send + Sync + 'static,
        Mutation: ObjectType + Send + Sync + 'static,
        Subscription: SubscriptionType + Send + Sync + 'static,
    {
        Response(schema.execute_batch(self.0).await)
    }
}

#[rocket::async_trait]
impl FromData for BatchRequest {
    type Error = ParseRequestError;

    async fn from_data(req: &rocket::Request<'_>, data: Data) -> data::Outcome<Self, Self::Error> {
        let opts: MultipartOptions = req.managed_state().copied().unwrap_or_default();

        let request = async_graphql::http::receive_batch_body(
            req.headers().get_one("Content-Type"),
            data.open(
                req.limits()
                    .get("graphql")
                    .unwrap_or_else(|| 128.kibibytes()),
            )
            .compat(),
            opts,
        )
        .await;

        match request {
            Ok(request) => data::Outcome::Success(Self(request)),
            Err(e) => data::Outcome::Failure((
                match e {
                    ParseRequestError::PayloadTooLarge => Status::PayloadTooLarge,
                    _ => Status::BadRequest,
                },
                e,
            )),
        }
    }
}

/// A GraphQL request which can be extracted from a query string or the request's body.
///
/// # Examples
///
/// ```ignore
/// #[rocket::post("/graphql?<query..>")]
/// async fn graphql_query(schema: State<'_, ExampleSchema>, query: Request) -> Result<Response, Status> {
///     query.execute(&schema).await
/// }
///
/// #[rocket::post("/graphql", data = "<request>", format = "application/json")]
/// async fn graphql_request(schema: State<'_, ExampleSchema>, request: Request) -> Result<Response, Status> {
///     request.execute(&schema).await
/// }
/// ```
#[derive(Debug)]
pub struct Request(pub async_graphql::Request);

impl Request {
    /// Shortcut method to execute the request on the schema.
    pub async fn execute<Query, Mutation, Subscription>(
        self,
        schema: &Schema<Query, Mutation, Subscription>,
    ) -> Response
    where
        Query: ObjectType + Send + Sync + 'static,
        Mutation: ObjectType + Send + Sync + 'static,
        Subscription: SubscriptionType + Send + Sync + 'static,
    {
        Response(schema.execute(self.0).await.into())
    }
}

impl<'q> FromQuery<'q> for Request {
    type Error = serde::de::value::Error;

    fn from_query(query: request::Query<'_>) -> Result<Self, Self::Error> {
        Ok(Self(async_graphql::Request::deserialize(
            QueryDeserializer(query),
        )?))
    }
}

#[rocket::async_trait]
impl FromData for Request {
    type Error = ParseRequestError;

    async fn from_data(req: &rocket::Request<'_>, data: Data) -> data::Outcome<Self, Self::Error> {
        BatchRequest::from_data(req, data)
            .await
            .and_then(|request| match request.0.into_single() {
                Ok(single) => data::Outcome::Success(Self(single)),
                Err(e) => data::Outcome::Failure((Status::BadRequest, e)),
            })
    }
}

/// Wrapper around `async-graphql::Response` that is a Rocket responder so it can be returned from
/// a routing function in Rocket.
///
/// It contains a `BatchResponse` but since a response is a type of batch response it works for
/// both.
#[derive(Debug)]
pub struct Response(pub async_graphql::BatchResponse);

impl From<async_graphql::BatchResponse> for Response {
    fn from(batch: async_graphql::BatchResponse) -> Self {
        Self(batch)
    }
}
impl From<async_graphql::Response> for Response {
    fn from(res: async_graphql::Response) -> Self {
        Self(res.into())
    }
}

impl<'r> Responder<'r, 'static> for Response {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> response::Result<'static> {
        let body = serde_json::to_string(&self.0).unwrap();

        let mut response = rocket::Response::new();

        if self.0.is_ok() {
            if let Some(cache_control) = self.0.cache_control().value() {
                response.set_header(Header::new("cache-control", cache_control));
            }
        }

        response.set_header(ContentType::new("application", "json"));
        response.set_sized_body(body.len(), Cursor::new(body));

        Ok(response)
    }
}
