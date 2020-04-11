use actix_web::{guard, web, App, HttpResponse, HttpServer};
use async_graphql::http::{playground_source, GQLRequest, GQLResponse, IntoQueryBuilder};
use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema, SimpleObject, ID};
use futures::TryFutureExt;

type MySchema = Schema<Query, EmptyMutation, EmptySubscription>;

#[SimpleObject]
struct User {
    #[field]
    id: ID,

    #[field]
    username: String,
}

struct Query;

#[Object(extends)]
impl Query {
    #[field]
    async fn me(&self) -> User {
        User {
            id: "1234".into(),
            username: "Me".to_string(),
        }
    }

    #[entity]
    async fn find_user_by_id(&self, id: ID) -> User {
        let username = if id == "1234" {
            "Me".to_string()
        } else {
            format!("User {}", id)
        };
        User { id, username }
    }
}

async fn index(s: web::Data<MySchema>, req: web::Json<GQLRequest>) -> web::Json<GQLResponse> {
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

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    HttpServer::new(move || {
        App::new()
            .data(schema.clone())
            .service(web::resource("/").guard(guard::Post()).to(index))
            .service(web::resource("/").guard(guard::Get()).to(gql_playgound))
    })
    .bind("127.0.0.1:4001")?
    .run()
    .await
}
