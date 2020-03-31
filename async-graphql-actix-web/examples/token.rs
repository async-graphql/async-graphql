use actix_web::{web, App, HttpServer};
use async_graphql::{Context, EmptyMutation, EmptySubscription, Schema};

struct MyToken(Option<String>);

struct QueryRoot;

#[async_graphql::Object]
impl QueryRoot {
    #[field]
    async fn current_token<'a>(&self, ctx: &'a Context<'_>) -> Option<&'a str> {
        ctx.data::<MyToken>().0.as_deref()
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
        let handler = async_graphql_actix_web::HandlerBuilder::new(schema)
            .enable_subscription()
            .enable_ui("http://localhost:8000", None)
            .on_request(|req, builder| {
                builder.data(MyToken(
                    req.headers()
                        .get("Token")
                        .and_then(|value| value.to_str().map(ToString::to_string).ok()),
                ))
            })
            .build();
        App::new().service(web::resource("/").to(handler))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
