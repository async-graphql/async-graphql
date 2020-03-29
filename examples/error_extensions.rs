use actix_rt;
use actix_web::{guard, web, App, HttpResponse, HttpServer};
use async_graphql::http::{graphiql_source, playground_source, GQLRequest, GQLResponse};
use async_graphql::*;
use serde_json::json;

#[derive(Debug)]
pub enum MyError {
    NotFound,
    ServerError(String),
    ErrorWithoutExtensions,
}

// Let's implement a mapping from our MyError to async_graphql::Error (anyhow::Error).
// But instead of mapping to async_graphql::Error directly we map to async_graphql::ExtendedError,
// which gives us the opportunity to provide custom extensions to our errors.
// Note: Values which can't get serialized to JSON-Objects simply get ignored.
impl From<MyError> for Error {
    fn from(my_error: MyError) -> Error {
        match my_error {
            MyError::NotFound => {
                let msg = "Could not find ressource".to_owned();
                let extensions = json!({"code": "NOT_FOUND"});
                ExtendedError(msg, extensions).into()
            }
            MyError::ServerError(reason) => {
                ExtendedError("ServerError".to_owned(), json!({ "reason": reason })).into()
            }

            MyError::ErrorWithoutExtensions => ExtendedError(
                "No Extensions".to_owned(),
                json!("This will be ignored since it does not represent an object."),
            )
            .into(),
        }
    }
}

fn get_my_error() -> std::result::Result<String, MyError> {
    Err(MyError::ServerError("The database is locked".to_owned()))
}

struct QueryRoot {}

#[Object]
impl QueryRoot {
    #[field]
    async fn do_not_find(&self) -> Result<i32> {
        Err(MyError::NotFound)?
    }

    #[field]
    async fn fail(&self) -> Result<String> {
        Ok(get_my_error()?)
    }

    #[field]
    async fn without_extensions(&self) -> Result<String> {
        Err(MyError::ErrorWithoutExtensions)?
    }

    // Using the ResultExt trait, we can attach extensions on the fly capturing the execution
    // environment. This method works on foreign types as well. The trait is implemented for all
    // Results where the error variant implements Display.
    #[field]
    async fn parse_value(&self, val: String) -> Result<i32> {
        val.parse().extend_err(|err| {
            json!({ "description": format!("Could not parse value {}: {}", val, err) })
        })
    }
}

async fn index(
    s: web::Data<Schema<QueryRoot, EmptyMutation, EmptySubscription>>,
    req: web::Json<GQLRequest>,
) -> web::Json<GQLResponse> {
    web::Json(req.into_inner().execute(&s).await)
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
            .data(Schema::new(QueryRoot {}, EmptyMutation, EmptySubscription))
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
