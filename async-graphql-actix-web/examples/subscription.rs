use actix::clock::Duration;
use actix_web::{web, App, HttpServer};
use async_graphql::{EmptyMutation, Schema};
use futures::{Stream, StreamExt};

struct QueryRoot;

#[async_graphql::Object]
impl QueryRoot {
    #[field]
    async fn value(&self) -> i32 {
        0
    }
}

struct SubscriptionRoot;

#[async_graphql::Subscription]
impl SubscriptionRoot {
    #[field]
    fn interval(&self, n: i32) -> impl Stream<Item = i32> {
        let mut value = 0;
        actix_rt::time::interval(Duration::from_secs(1)).map(move |_| {
            value += n;
            value
        })
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        let schema = Schema::new(QueryRoot, EmptyMutation, SubscriptionRoot);
        let handler = async_graphql_actix_web::HandlerBuilder::new(schema)
            .enable_ui("http://localhost:8000", Some("ws://localhost:8000"))
            .enable_subscription()
            .build();
        App::new().service(web::resource("/").to(handler))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
