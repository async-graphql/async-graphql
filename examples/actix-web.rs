mod starwars;

use actix_web::{guard, web, App, HttpResponse, HttpServer};
use async_graphql::http::{graphiql_source, playground_source};
use async_graphql::{GQLEmptyMutation, Schema};

async fn gql_playgound() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source("/"))
}

async fn gql_graphiql() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(graphiql_source("/"))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        let schema =
            Schema::new(starwars::QueryRoot, GQLEmptyMutation).data(starwars::StarWars::new());
        let handler = async_graphql_actix_web::HandlerBuilder::new(schema).build();

        App::new()
            .service(web::resource("/").to(handler))
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
