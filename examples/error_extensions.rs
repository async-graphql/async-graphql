#[macro_use]
extern crate thiserror;

use actix_rt;
use actix_web::{guard, web, App, HttpResponse, HttpServer};
use async_graphql::http::{graphiql_source, playground_source, GQLRequest, GQLResponse};
use async_graphql::*;
use futures::TryFutureExt;
use serde_json::json;

#[derive(Debug, Error)]
pub enum MyError {
    #[error("Could not find resource")]
    NotFound,

    #[error("ServerError")]
    ServerError(String),

    #[error("No Extensions")]
    ErrorWithoutExtensions,
}

impl MyError {
    fn extend_err(&self) -> serde_json::Value {
        match self {
            MyError::NotFound => json!({"code": "NOT_FOUND"}),
            MyError::ServerError(reason) => json!({ "reason": reason }),
            MyError::ErrorWithoutExtensions => {
                json!("This will be ignored since it does not represent an object.")
            }
        }
    }
}

fn get_my_error() -> std::result::Result<String, MyError> {
    Err(MyError::ServerError("The database is locked".to_owned()))
}

struct QueryRoot;

#[Object]
impl QueryRoot {
    #[field]
    async fn do_not_find(&self) -> FieldResult<i32> {
        Err(MyError::NotFound).extend_err(MyError::extend_err)
    }

    #[field]
    async fn fail(&self) -> FieldResult<String> {
        Ok(get_my_error().extend_err(MyError::extend_err)?)
    }

    #[field]
    async fn without_extensions(&self) -> FieldResult<String> {
        Err(MyError::ErrorWithoutExtensions).extend_err(MyError::extend_err)?
    }

    // Using the ResultExt trait, we can attach extensions on the fly capturing the execution
    // environment. This method works on foreign types as well. The trait is implemented for all
    // Results where the error variant implements `std::error::Error`.
    #[field]
    async fn parse_value(&self, val: String) -> FieldResult<i32> {
        val.parse().extend_err(|err| {
            json!({ "description": format!("Could not parse value '{}': {}", val, err) })
        })
    }

    #[field]
    async fn parse_value2(&self, val: String) -> FieldResult<i32> {
        Ok(val.parse()?)
    }
}

async fn index(
    s: web::Data<Schema<QueryRoot, EmptyMutation, EmptySubscription>>,
    req: web::Json<GQLRequest>,
) -> web::Json<GQLResponse> {
    web::Json(GQLResponse(
        req.into_inner()
            .into_query_builder(&s)
            .and_then(|builder| builder.execute())
            .await,
    ))
}

async fn gql_playgound() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source("/", None))
}

async fn gql_graphiql() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(graphiql_source("/"))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .data(Schema::new(QueryRoot, EmptyMutation, EmptySubscription))
            .service(web::resource("/").guard(guard::Post()).to(index))
            .service(web::resource("/").guard(guard::Get()).to(gql_playgound))
            .service(
                web::resource("/graphiql")
                    .guard(guard::Get())
                    .to(gql_graphiql),
            )
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
