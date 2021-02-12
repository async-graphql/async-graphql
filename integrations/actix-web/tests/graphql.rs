mod test_utils;
use actix_web::{guard, test, web, App};
use async_graphql::*;
use serde_json::json;
use test_utils::*;

#[actix_rt::test]
async fn test_playground() {
    let srv = test::start(|| {
        App::new().service(
            web::resource("/")
                .guard(guard::Get())
                .to(test_utils::gql_playgound),
        )
    });
    let mut response = srv.get("/").send().await.unwrap();
    assert!(response.status().is_success());
    let body = response.body().await.unwrap();
    assert!(std::str::from_utf8(&body).unwrap().contains("graphql"));
}

#[actix_rt::test]
async fn test_add() {
    let srv = test::start(|| {
        App::new()
            .data(Schema::new(AddQueryRoot, EmptyMutation, EmptySubscription))
            .service(
                web::resource("/")
                    .guard(guard::Post())
                    .to(gql_handle_schema::<AddQueryRoot, EmptyMutation, EmptySubscription>),
            )
    });
    let mut response = srv
        .post("/")
        .send_body(r#"{"query":"{ add(a: 10, b: 20) }"}"#)
        .await
        .unwrap();
    assert!(response.status().is_success());
    let body = response.body().await.unwrap();
    assert_eq!(body, json!({"data": {"add": 30}}).to_string());
}

#[actix_rt::test]
async fn test_hello() {
    let srv = test::start(|| {
        App::new()
            .data(Schema::new(
                HelloQueryRoot,
                EmptyMutation,
                EmptySubscription,
            ))
            .service(
                web::resource("/")
                    .guard(guard::Post())
                    .to(gql_handle_schema::<HelloQueryRoot, EmptyMutation, EmptySubscription>),
            )
    });

    let mut response = srv
        .post("/")
        .send_body(r#"{"query":"{ hello }"}"#)
        .await
        .unwrap();
    assert!(response.status().is_success());
    let body = response.body().await.unwrap();
    assert_eq!(
        body,
        json!({"data": {"hello": "Hello, world!"}}).to_string()
    );
}

#[actix_rt::test]
async fn test_hello_header() {
    let srv = test::start(|| {
        App::new()
            .data(Schema::new(
                HelloQueryRoot,
                EmptyMutation,
                EmptySubscription,
            ))
            .service(
                web::resource("/")
                    .guard(guard::Post())
                    .to(gql_handle_schema_with_header::<HelloQueryRoot>),
            )
    });

    let mut response = srv
        .post("/")
        .append_header(("Name", "Foo"))
        .send_body(r#"{"query":"{ hello }"}"#)
        .await
        .unwrap();
    assert!(response.status().is_success());
    let body = response.body().await.unwrap();
    assert_eq!(body, json!({"data": {"hello": "Hello, Foo!"}}).to_string());
}

#[actix_rt::test]
async fn test_count() {
    let srv = test::start(|| {
        App::new()
            .data(
                Schema::build(CountQueryRoot, CountMutation, EmptySubscription)
                    .data(Count::default())
                    .finish(),
            )
            .service(
                web::resource("/")
                    .guard(guard::Post())
                    .to(gql_handle_schema::<CountQueryRoot, CountMutation, EmptySubscription>),
            )
    });
    count_action_helper(0, r#"{"query":"{ count }"}"#, &srv).await;
    count_action_helper(10, r#"{"query":"mutation{ addCount(count: 10) }"}"#, &srv).await;
    count_action_helper(
        8,
        r#"{"query":"mutation{ subtractCount(count: 2) }"}"#,
        &srv,
    )
    .await;
    count_action_helper(
        6,
        r#"{"query":"mutation{ subtractCount(count: 2) }"}"#,
        &srv,
    )
    .await;
}

async fn count_action_helper(expected: i32, body: &'static str, srv: &test::TestServer) {
    let mut response = srv.post("/").send_body(body).await.unwrap();
    assert!(response.status().is_success());
    let body = response.body().await.unwrap();
    assert!(std::str::from_utf8(&body)
        .unwrap()
        .contains(&expected.to_string()));
}
