//! Async-graphql integration with Rocket.
//!
//! Note: This integrates with the unreleased version 0.5 of Rocket, and so
//! breaking changes in both this library and Rocket are to be expected.
//!
//! To configure options for sending and receiving multipart requests, add your
//! instance of `MultipartOptions` to the state managed by Rocket
//! (`.manage(your_multipart_options)`).
//!
//! **[Full Example](<https://github.com/async-graphql/examples/blob/master/rocket/starwars/src/main.rs>)**

#![warn(missing_docs)]
#![forbid(unsafe_code)]

use core::any::Any;
use std::io::Cursor;

use async_graphql::{http::MultipartOptions, Executor, ParseRequestError};
use rocket::{
    data::{self, Data, FromData, ToByteUnit},
    form::FromForm,
    http::{ContentType, Header, Status},
    response::{self, Responder},
};
use tokio_util::compat::TokioAsyncReadCompatExt;

/// A batch request which can be extracted from a request's body.
///
/// # Examples
///
/// ```ignore
/// #[rocket::post("/graphql", data = "<request>", format = "application/json", rank = 1)]
/// async fn graphql_request(schema: State<'_, ExampleSchema>, request: BatchRequest) -> Response {
///     request.execute(&schema).await
/// }
/// ```
#[derive(Debug)]
pub struct GraphQLBatchRequest(pub async_graphql::BatchRequest);

impl GraphQLBatchRequest {
    /// Shortcut method to execute the request on the executor.
    pub async fn execute<E>(self, executor: &E) -> GraphQLResponse
    where
        E: Executor,
    {
        GraphQLResponse(executor.execute_batch(self.0).await)
    }
}

#[rocket::async_trait]
impl<'r> FromData<'r> for GraphQLBatchRequest {
    type Error = ParseRequestError;

    async fn from_data(req: &'r rocket::Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {
        let opts: MultipartOptions = req.rocket().state().copied().unwrap_or_default();

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
            Err(e) => data::Outcome::Error((
                match e {
                    ParseRequestError::PayloadTooLarge => Status::PayloadTooLarge,
                    _ => Status::BadRequest,
                },
                e,
            )),
        }
    }
}

/// A GraphQL request which can be extracted from the request's body.
///
/// # Examples
///
/// ```ignore
/// #[rocket::post("/graphql", data = "<request>", format = "application/json", rank = 2)]
/// async fn graphql_request(schema: State<'_, ExampleSchema>, request: Request) -> Result<Response, Status> {
///     request.execute(&schema).await
/// }
/// ```
#[derive(Debug)]
pub struct GraphQLRequest(pub async_graphql::Request);

impl GraphQLRequest {
    /// Shortcut method to execute the request on the schema.
    pub async fn execute<E>(self, executor: &E) -> GraphQLResponse
    where
        E: Executor,
    {
        GraphQLResponse(executor.execute(self.0).await.into())
    }

    /// Insert some data for this request.
    #[must_use]
    pub fn data<D: Any + Send + Sync>(mut self, data: D) -> Self {
        self.0.data.insert(data);
        self
    }
}

impl From<GraphQLQuery> for GraphQLRequest {
    fn from(query: GraphQLQuery) -> Self {
        let mut request = async_graphql::Request::new(query.query);

        if let Some(operation_name) = query.operation_name {
            request = request.operation_name(operation_name);
        }

        if let Some(variables) = query.variables {
            let value = serde_json::from_str(&variables).unwrap_or_default();
            let variables = async_graphql::Variables::from_json(value);
            request = request.variables(variables);
        }

        GraphQLRequest(request)
    }
}

/// A GraphQL request which can be extracted from a query string.
///
/// # Examples
///
/// ```ignore
/// #[rocket::get("/graphql?<query..>")]
/// async fn graphql_query(schema: State<'_, ExampleSchema>, query: Query) -> Result<Response, Status> {
///     query.execute(&schema).await
/// }
/// ```
#[derive(FromForm, Debug)]
pub struct GraphQLQuery {
    query: String,
    #[field(name = "operationName")]
    operation_name: Option<String>,
    variables: Option<String>,
}

impl GraphQLQuery {
    /// Shortcut method to execute the request on the schema.
    pub async fn execute<E>(self, executor: &E) -> GraphQLResponse
    where
        E: Executor,
    {
        let request: GraphQLRequest = self.into();
        request.execute(executor).await
    }
}

#[rocket::async_trait]
impl<'r> FromData<'r> for GraphQLRequest {
    type Error = ParseRequestError;

    async fn from_data(req: &'r rocket::Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {
        GraphQLBatchRequest::from_data(req, data)
            .await
            .and_then(|request| match request.0.into_single() {
                Ok(single) => data::Outcome::Success(Self(single)),
                Err(e) => data::Outcome::Error((Status::BadRequest, e)),
            })
    }
}

/// Wrapper around `async-graphql::Response` that is a Rocket responder so it
/// can be returned from a routing function in Rocket.
///
/// It contains a `BatchResponse` but since a response is a type of batch
/// response it works for both.
#[derive(Debug)]
pub struct GraphQLResponse(pub async_graphql::BatchResponse);

impl From<async_graphql::BatchResponse> for GraphQLResponse {
    fn from(batch: async_graphql::BatchResponse) -> Self {
        Self(batch)
    }
}
impl From<async_graphql::Response> for GraphQLResponse {
    fn from(res: async_graphql::Response) -> Self {
        Self(res.into())
    }
}

impl<'r> Responder<'r, 'static> for GraphQLResponse {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> response::Result<'static> {
        let body = serde_json::to_string(&self.0).unwrap();

        let mut response = rocket::Response::new();
        response.set_header(ContentType::new("application", "json"));

        if self.0.is_ok() {
            if let Some(cache_control) = self.0.cache_control().value() {
                response.set_header(Header::new("cache-control", cache_control));
            }
        }

        for (name, value) in self.0.http_headers_iter() {
            if let Ok(value) = value.to_str() {
                response.adjoin_header(Header::new(name.as_str().to_string(), value.to_string()));
            }
        }

        response.set_sized_body(body.len(), Cursor::new(body));

        Ok(response)
    }
}
