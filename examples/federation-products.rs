use actix_web::{guard, web, App, HttpResponse, HttpServer};
use async_graphql::http::{playground_source, GQLRequest, GQLResponse, IntoQueryBuilder};
use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject};
use futures::TryFutureExt;

type MySchema = Schema<Query, EmptyMutation, EmptySubscription>;

#[SimpleObject]
struct Product {
    #[field]
    upc: String,

    #[field]
    name: String,

    #[field]
    price: i32,
}

struct Query;

#[Object(extends)]
impl Query {
    #[field]
    async fn top_products<'a>(&self, ctx: &'a Context<'_>) -> &'a Vec<Product> {
        ctx.data::<Vec<Product>>()
    }

    #[entity]
    async fn find_product_by_upc<'a>(
        &self,
        ctx: &'a Context<'_>,
        upc: String,
    ) -> Option<&'a Product> {
        let hats = ctx.data::<Vec<Product>>();
        hats.iter().find(|product| product.upc == upc)
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
    let hats = vec![
        Product {
            upc: "top-1".to_string(),
            name: "Trilby".to_string(),
            price: 11,
        },
        Product {
            upc: "top-2".to_string(),
            name: "Fedora".to_string(),
            price: 22,
        },
        Product {
            upc: "top-3".to_string(),
            name: "Boater".to_string(),
            price: 33,
        },
    ];

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(hats)
        .finish();

    HttpServer::new(move || {
        App::new()
            .data(schema.clone())
            .service(web::resource("/").guard(guard::Post()).to(index))
            .service(web::resource("/").guard(guard::Get()).to(gql_playgound))
    })
    .bind("127.0.0.1:4002")?
    .run()
    .await
}
