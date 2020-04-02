#[macro_use]
extern crate thiserror;

use actix_rt;
use actix_web::{guard, web, App, HttpResponse, HttpServer};
use async_graphql::http::{graphiql_source, playground_source, GQLRequest, GQLResponse};
use async_graphql::ErrorExtensions;
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

impl ErrorExtensions for MyError {
    // lets define our base extensions
    fn extend(&self) -> FieldError {
        let extensions = match self {
            MyError::NotFound => json!({"code": "NOT_FOUND"}),
            MyError::ServerError(reason) => json!({ "reason": reason }),
            MyError::ErrorWithoutExtensions => {
                json!("This will be ignored since it does not represent an object.")
            }
        };

        FieldError(format!("{}", self), Some(extensions))
    }
}

struct QueryRoot;

#[Object]
impl QueryRoot {
    // It works on foreign types without extensions as before
    #[field]
    async fn parse_without_extensions(&self) -> FieldResult<i32> {
        Ok("234a".parse()?)
    }

    // Foreign types can be extended
    #[field]
    async fn parse_with_extensions(&self) -> FieldResult<i32> {
        Ok("234a"
            .parse()
            .map_err(|e: std::num::ParseIntError| e.extend_with(|_| json!({"code": 404})))?)
    }

    // THIS does unfortunately NOT work because ErrorExtensions is implemented for &E and not E.
    // Which is necessary for the overwrite by the user.

    //#[field]
    // async fn parse_with_extensions_result(&self) -> FieldResult<i32> {
    //    Ok("234a".parse().extend_err(|_| json!({"code": 404}))?)
    // }

    // Using our own types we can implement some base extensions
    #[field]
    async fn extend(&self) -> FieldResult<i32> {
        Err(MyError::NotFound.extend())?
    }

    // Or on the result
    #[field]
    async fn extend_result(&self) -> FieldResult<i32> {
        Err(MyError::NotFound).extend()?
    }

    // Base extensions can be further extended
    #[field]
    async fn more_extensions(&self) -> FieldResult<String> {
        // resolves to extensions: { "code": "NOT_FOUND", "reason": "my reason" }
        Err(MyError::NotFound.extend_with(|_e| json!({"reason": "my reason"})))?
    }

    // works with results as well
    #[field]
    async fn more_extensions_on_result(&self) -> FieldResult<String> {
        // resolves to extensions: { "code": "NOT_FOUND", "reason": "my reason" }
        Err(MyError::NotFound).extend_err(|_e| json!({"reason": "my reason"}))?
    }

    // extend_with is chainable
    #[field]
    async fn chainable_extensions(&self) -> FieldResult<String> {
        let err = MyError::NotFound
            .extend_with(|_| json!({"ext1": 1}))
            .extend_with(|_| json!({"ext2": 2}))
            .extend_with(|_| json!({"ext3": 3}));
        Err(err)?
    }

    // extend_with overwrites keys which are already present
    #[field]
    async fn overwrite(&self) -> FieldResult<String> {
        Err(MyError::NotFound.extend_with(|_| json!({"code": "overwritten"})))?
    }
}

async fn index(
    s: web::Data<Schema<QueryRoot, EmptyMutation, EmptySubscription>>,
    req: web::Json<GQLRequest>,
) -> web::Json<GQLResponse> {
    web::Json(GQLResponse(
        futures::future::ready(req.into_inner().into_query_builder(&s))
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
    println!("Playground: http://localhost:8000");
    println!("Graphiql: http://localhost:8000/graphiql");

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
