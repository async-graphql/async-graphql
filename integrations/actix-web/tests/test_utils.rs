use actix_web::{web, HttpRequest, HttpResponse};
use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    Context, EmptyMutation, EmptySubscription, Object, ObjectType, Schema, SubscriptionType,
};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use async_mutex::Mutex;

pub async fn gql_playgound() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(GraphQLPlaygroundConfig::new("/")))
}

pub(crate) struct AddQueryRoot;

#[Object]
impl AddQueryRoot {
    /// Returns the sum of a and b
    async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }
}

struct Hello(String);

pub(crate) struct HelloQueryRoot;

#[Object]
impl HelloQueryRoot {
    /// Returns hello
    async fn hello<'a>(&self, ctx: &'a Context<'_>) -> String {
        let name = ctx.data_opt::<Hello>().map(|hello| hello.0.as_str());
        format!("Hello, {}!", name.unwrap_or("world"))
    }
}

pub type Count = Mutex<i32>;

pub(crate) struct CountQueryRoot;

#[Object]
impl CountQueryRoot {
    async fn count<'a>(&self, ctx: &'a Context<'_>) -> i32 {
        *ctx.data_unchecked::<Count>().lock().await
    }
}

pub(crate) struct CountMutation;

#[Object]
impl CountMutation {
    async fn add_count<'a>(&self, ctx: &'a Context<'_>, count: i32) -> i32 {
        let mut guard_count = ctx.data_unchecked::<Count>().lock().await;
        *guard_count += count;
        *guard_count
    }

    async fn subtract_count<'a>(&self, ctx: &'a Context<'_>, count: i32) -> i32 {
        let mut guard_count = ctx.data_unchecked::<Count>().lock().await;
        *guard_count -= count;
        *guard_count
    }
}

pub async fn gql_handle_schema<
    Q: ObjectType + 'static,
    M: ObjectType + 'static,
    S: SubscriptionType + 'static,
>(
    schema: web::Data<Schema<Q, M, S>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

pub async fn gql_handle_schema_with_header<T: ObjectType + 'static>(
    schema: actix_web::web::Data<Schema<T, EmptyMutation, EmptySubscription>>,
    req: HttpRequest,
    gql_request: GraphQLRequest,
) -> GraphQLResponse {
    let name = req
        .headers()
        .get("Name")
        .and_then(|value| value.to_str().map(|s| Hello(s.to_string())).ok());
    let mut request = gql_request.into_inner();
    if let Some(name) = name {
        request = request.data(name);
    }
    schema.execute(request).await.into()
}
