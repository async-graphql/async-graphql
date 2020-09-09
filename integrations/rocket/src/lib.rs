//! Async-graphql integration with Rocket

#![warn(missing_docs)]
#![forbid(unsafe_code)]

use async_graphql::{
    IntoQueryBuilder, IntoQueryBuilderOpts, ObjectType, QueryBuilder, QueryResponse, Schema,
    SubscriptionType, Variables,
};
use log::{error, info};
use rocket::{
    data::{self, FromData},
    data::{Data, ToByteUnit},
    fairing::{AdHoc, Fairing},
    http::{ContentType, Header, Status},
    request::{self, FromQuery, Outcome},
    response::{self, Responder, ResponseBuilder},
    Request, Response, State,
};
use std::{io::Cursor, sync::Arc};
use tokio_util::compat::Tokio02AsyncReadCompatExt;
use yansi::Paint;

/// Contains the fairing functions, to attach GraphQL with the desired `Schema`, and optionally
/// `QueryBuilderOpts`, to Rocket.
///
/// # Examples
/// **[Full Example](<https://github.com/async-graphql/examples/blob/master/rocket/starwars/src/main.rs>)**
///
/// ```rust,no_run
///
/// use async_graphql::{EmptyMutation, EmptySubscription, Schema, Object};
/// use async_graphql_rocket::{GQLRequest, GraphQL, GQLResponse};
/// use rocket::{response::content, routes, State, http::Status};
///
/// type ExampleSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;
/// struct QueryRoot;
///
/// #[Object]
/// impl QueryRoot {
///     #[field(desc = "Returns the sum of a and b")]
///     async fn add(&self, a: i32, b: i32) -> i32 {
///          a + b
///     }
/// }
///
/// #[rocket::post("/?<query..>")]
/// async fn graphql_query(schema: State<'_, ExampleSchema>, query: GQLRequest) -> Result<GQLResponse, Status> {
///     query.execute(&schema)
///         .await
/// }
///
/// #[rocket::post("/", data = "<request>", format = "application/json")]
/// async fn graphql_request(schema: State<'_, ExampleSchema>, request: GQLRequest) -> Result<GQLResponse, Status> {
///     request.execute(&schema)
///         .await
/// }
///
/// #[rocket::launch]
/// fn rocket() -> rocket::Rocket {
///     let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
///     rocket::ignite()
///         .attach(GraphQL::fairing(schema))
///         .mount("/", routes![graphql_query, graphql_request])
/// }
/// ```
pub struct GraphQL;

impl GraphQL {
    /// Fairing with default `QueryBuilderOpts`. You just need to pass in your `Schema` and then can
    /// attach the `Fairing` to Rocket.
    ///
    /// # Examples
    ///
    /// ```rust,no_run,ignore
    ///     rocket::ignite()
    ///         .attach(GraphQL::fairing(schema))
    ///         .mount("/", routes![graphql_query, graphql_request])
    /// ```
    pub fn fairing<Q, M, S>(schema: Schema<Q, M, S>) -> impl Fairing
    where
        Q: ObjectType + Send + Sync + 'static,
        M: ObjectType + Send + Sync + 'static,
        S: SubscriptionType + Send + Sync + 'static,
    {
        GraphQL::attach(schema, Default::default())
    }

    /// Fairing to which you need to pass `QueryBuilderOpts` and your `Schema`. Then you can
    /// attach the `Fairing` to Rocket.
    ///
    /// # Examples
    ///
    /// ```rust,no_run,ignore
    ///     let opts: IntoQueryBuilderOpts = Default::default();
    ///     rocket::ignite()
    ///         .attach(GraphQL::fairing_with_opts(schema, opts))
    ///         .mount("/", routes![graphql_query, graphql_request])
    /// ```
    pub fn fairing_with_opts<Q, M, S>(
        schema: Schema<Q, M, S>,
        opts: IntoQueryBuilderOpts,
    ) -> impl Fairing
    where
        Q: ObjectType + Send + Sync + 'static,
        M: ObjectType + Send + Sync + 'static,
        S: SubscriptionType + Send + Sync + 'static,
    {
        GraphQL::attach(schema, opts)
    }

    fn attach<Q, M, S>(schema: Schema<Q, M, S>, opts: IntoQueryBuilderOpts) -> impl Fairing
    where
        Q: ObjectType + Send + Sync + 'static,
        M: ObjectType + Send + Sync + 'static,
        S: SubscriptionType + Send + Sync + 'static,
    {
        AdHoc::on_attach("GraphQL", move |rocket| async move {
            let emoji = if cfg!(windows) { "" } else { "📄 " };
            info!(
                "{}{}",
                Paint::masked(emoji),
                Paint::magenta(format!("GraphQL {}:", Paint::blue(""))).wrap()
            );

            Ok(rocket.manage(schema).manage(Arc::new(opts)))
        })
    }
}

/// Implements `FromQuery` and `FromData`, so that it can be used as parameter in a
/// Rocket route.
///
/// # Examples
///
/// ```rust,no_run,ignore
/// #[rocket::post("/?<query..>")]
/// async fn graphql_query(schema: State<'_, ExampleSchema>, query: GQLRequest) -> Result<GQLResponse, Status> {
///     query.execute(&schema)
///         .await
/// }
///
/// #[rocket::post("/", data = "<request>", format = "application/json")]
/// async fn graphql_request(schema: State<'_, ExampleSchema>, request: GQLRequest) -> Result<GQLResponse, Status> {
///     request.execute(&schema)
///         .await
/// }
/// ```
pub struct GQLRequest(pub QueryBuilder);

impl GQLRequest {
    /// Mimics `async_graphqlquery::QueryBuilder.execute()`.
    /// Executes the query, always return a complete result.
    pub async fn execute<Q, M, S>(self, schema: &Schema<Q, M, S>) -> Result<GQLResponse, Status>
    where
        Q: ObjectType + Send + Sync + 'static,
        M: ObjectType + Send + Sync + 'static,
        S: SubscriptionType + Send + Sync + 'static,
    {
        self.0.execute(schema).await.map(GQLResponse).map_err(|e| {
            error!("{}", e);
            Status::BadRequest
        })
    }
}

impl<'q> FromQuery<'q> for GQLRequest {
    type Error = String;

    fn from_query(query_items: request::Query) -> Result<Self, Self::Error> {
        let mut query = None;
        let mut operation_name = None;
        let mut variables = None;

        for query_item in query_items {
            let (key, value) = query_item.key_value();
            match key.as_str() {
                "query" => {
                    if query.is_some() {
                        return Err(r#"Multiple parameters named "query" found. Only one parameter by that name is allowed."#.to_string());
                    } else {
                        query = value.url_decode().map_err(|e| e.to_string())?.into();
                    }
                }
                "operation_name" => {
                    if operation_name.is_some() {
                        return Err(r#"Multiple parameters named "operation_name" found. Only one parameter by that name is allowed."#.to_string());
                    } else {
                        operation_name = value.url_decode().map_err(|e| e.to_string())?.into();
                    }
                }
                "variables" => {
                    if variables.is_some() {
                        return Err(r#"Multiple parameters named "variables" found. Only one parameter by that name is allowed."#.to_string());
                    } else {
                        let decoded = value.url_decode().map_err(|e| e.to_string())?;
                        let json_value = serde_json::from_str::<serde_json::Value>(&decoded)
                            .map_err(|e| e.to_string())?;
                        variables = Variables::parse_from_json(json_value).into();
                    }
                }
                _ => {
                    return Err(format!(
                        r#"Extra parameter named "{}" found. Extra parameters are not allowed."#,
                        key
                    ));
                }
            }
        }

        if let Some(query_source) = query {
            let mut builder = QueryBuilder::new(query_source);

            if let Some(variables) = variables {
                builder = builder.variables(variables);
            }

            if let Some(operation_name) = operation_name {
                builder = builder.operation_name(operation_name);
            }

            Ok(GQLRequest(builder))
        } else {
            Err(r#"Parameter "query" missing from request."#.to_string())
        }
    }
}

#[rocket::async_trait]
impl FromData for GQLRequest {
    type Error = String;

    async fn from_data(req: &Request<'_>, data: Data) -> data::Outcome<Self, Self::Error> {
        let opts = match req.guard::<State<'_, Arc<IntoQueryBuilderOpts>>>().await {
            Outcome::Success(opts) => opts,
            Outcome::Failure(_) => {
                return data::Outcome::Failure((
                    Status::InternalServerError,
                    "Missing IntoQueryBuilderOpts in State".to_string(),
                ))
            }
            Outcome::Forward(()) => unreachable!(),
        };

        let limit = req.limits().get("graphql");
        let stream = data.open(limit.unwrap_or_else(|| 128.kibibytes()));
        let builder = (req.headers().get_one("Content-Type"), stream.compat())
            .into_query_builder_opts(&opts)
            .await;

        match builder {
            Ok(builder) => data::Outcome::Success(GQLRequest(builder)),
            Err(e) => data::Outcome::Failure((Status::BadRequest, format!("{}", e))),
        }
    }
}

/// Wrapper around `async-graphql::query::QueryResponse` for implementing the trait
/// `rocket::response::responder::Responder`, so that `GQLResponse` can directly be returned
/// from a Rocket Route function.
pub struct GQLResponse(pub QueryResponse);

impl<'r> Responder<'r, 'static> for GQLResponse {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let gql_resp = async_graphql::http::GQLResponse(Ok(self.0));
        let body = serde_json::to_string(&gql_resp).unwrap();

        Response::build()
            .header(ContentType::new("application", "json"))
            .status(Status::Ok)
            .sized_body(body.len(), Cursor::new(body))
            .cache_control(&gql_resp.0)
            .ok()
    }
}

/// Extension trait, to allow the use of `cache_control` with for example `ResponseBuilder`.
pub trait CacheControl {
    /// Add the `async-graphql::query::QueryResponse` cache control value as header to the Rocket response.
    fn cache_control(&mut self, resp: &async_graphql::Result<QueryResponse>) -> &mut Self;
}

impl<'r> CacheControl for ResponseBuilder<'r> {
    fn cache_control(
        &mut self,
        resp: &async_graphql::Result<QueryResponse>,
    ) -> &mut ResponseBuilder<'r> {
        match resp {
            Ok(resp) if resp.cache_control.value().is_some() => self.header(Header::new(
                "cache-control",
                resp.cache_control.value().unwrap(),
            )),
            _ => self,
        }
    }
}
