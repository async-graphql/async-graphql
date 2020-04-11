use actix_web::{guard, web, App, HttpResponse, HttpServer};
use async_graphql::http::{playground_source, GQLRequest, GQLResponse, IntoQueryBuilder};
use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject, ID};
use futures::TryFutureExt;

type MySchema = Schema<Query, EmptyMutation, EmptySubscription>;

struct User {
    id: ID,
}

#[Object(extends)]
impl User {
    #[field(external)]
    async fn id(&self) -> &ID {
        &self.id
    }

    #[field]
    async fn reviews<'a>(&self, ctx: &'a Context<'_>) -> Vec<&'a Review> {
        let reviews = ctx.data::<Vec<Review>>();
        reviews
            .iter()
            .filter(|review| review.author.id == self.id)
            .collect()
    }
}

struct Product {
    upc: String,
}

#[Object(extends)]
impl Product {
    #[field(external)]
    async fn upc(&self) -> &String {
        &self.upc
    }

    #[field]
    async fn reviews<'a>(&self, ctx: &'a Context<'_>) -> Vec<&'a Review> {
        let reviews = ctx.data::<Vec<Review>>();
        reviews
            .iter()
            .filter(|review| review.product.upc == self.upc)
            .collect()
    }
}

#[SimpleObject]
struct Review {
    #[field]
    body: String,

    #[field(provides = "username")]
    author: User,

    #[field]
    product: Product,
}

struct Query;

#[Object]
impl Query {
    #[entity]
    async fn find_user_by_id<'a>(&self, id: ID) -> User {
        User { id }
    }

    #[entity]
    async fn find_product_by_upc<'a>(&self, upc: String) -> Product {
        Product { upc }
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
    let reviews = vec![
        Review {
            body: "A highly effective form of birth control.".into(),
            author: User { id: "1234".into() },
            product: Product {
                upc: "top-1".to_string(),
            },
        },
        Review {
            body: "Fedoras are one of the most fashionable hats around and can look great with a variety of outfits.".into(),
            author: User { id: "1234".into() },
            product: Product {
                upc: "top-1".to_string(),
            },
        },
        Review {
            body: "This is the last straw. Hat you will wear. 11/10".into(),
            author: User { id: "7777".into() },
            product: Product {
                upc: "top-1".to_string(),
            },
        },
    ];

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(reviews)
        .finish();

    HttpServer::new(move || {
        App::new()
            .data(schema.clone())
            .service(web::resource("/").guard(guard::Post()).to(index))
            .service(web::resource("/").guard(guard::Get()).to(gql_playgound))
    })
    .bind("127.0.0.1:4003")?
    .run()
    .await
}
