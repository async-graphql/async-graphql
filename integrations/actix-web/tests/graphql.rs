mod test_utils;
use actix_http::header;
use actix_web::{
    dev::{Service, ServiceResponse},
    guard, test,
    web::{self},
    App,
};
use async_graphql::*;
use serde_json::json;
use test_utils::*;

#[actix_rt::test]
async fn test_playground() {
    let app = test::init_service(
        App::new().service(
            web::resource("/")
                .guard(guard::Get())
                .to(test_utils::gql_playgound),
        ),
    )
    .await;

    let response = test::TestRequest::get().uri("/").send_request(&app).await;
    assert!(response.status().is_success());
    let body = test::read_body(response).await;
    assert!(std::str::from_utf8(&body).unwrap().contains("graphql"));
}

#[actix_rt::test]
async fn test_add() {
    let mut app = test::init_service(
        App::new()
            .app_data(actix_web::web::Data::new(Schema::new(
                AddQueryRoot,
                EmptyMutation,
                EmptySubscription,
            )))
            .service(
                web::resource("/")
                    .guard(guard::Post())
                    .to(gql_handle_schema::<AddQueryRoot, EmptyMutation, EmptySubscription>),
            ),
    )
    .await;

    let response = test::TestRequest::post()
        .uri("/")
        .append_header((header::CONTENT_TYPE, "application/json"))
        .set_payload(r#"{"query":"{ add(a: 10, b: 20) }"}"#)
        .send_request(&mut app)
        .await;

    assert!(response.status().is_success());
    let body = test::read_body(response).await;
    println!("Body: {:?}", body);
    assert_eq!(body, json!({"data": {"add": 30}}).to_string());
}

#[actix_rt::test]
async fn test_hello() {
    let app = test::init_service(
        App::new()
            .app_data(actix_web::web::Data::new(Schema::new(
                HelloQueryRoot,
                EmptyMutation,
                EmptySubscription,
            )))
            .service(
                web::resource("/")
                    .guard(guard::Post())
                    .to(gql_handle_schema::<HelloQueryRoot, EmptyMutation, EmptySubscription>),
            ),
    )
    .await;

    let response = test::TestRequest::post()
        .uri("/")
        .append_header((header::CONTENT_TYPE, "application/json"))
        .set_payload(r#"{"query":"{ hello }"}"#)
        .send_request(&app)
        .await;

    assert!(response.status().is_success());
    let body = test::read_body(response).await;
    assert_eq!(
        body,
        json!({"data": {"hello": "Hello, world!"}}).to_string()
    );
}

#[actix_rt::test]
async fn test_hello_header() {
    let app = test::init_service(
        App::new()
            .app_data(actix_web::web::Data::new(Schema::new(
                HelloQueryRoot,
                EmptyMutation,
                EmptySubscription,
            )))
            .service(
                web::resource("/")
                    .guard(guard::Post())
                    .to(gql_handle_schema_with_header::<HelloQueryRoot>),
            ),
    )
    .await;

    let response = test::TestRequest::post()
        .uri("/")
        .append_header((header::CONTENT_TYPE, "application/json"))
        .append_header(("Name", "Foo"))
        .set_payload(r#"{"query":"{ hello }"}"#)
        .send_request(&app)
        .await;

    assert!(response.status().is_success());
    let body = test::read_body(response).await;
    assert_eq!(body, json!({"data": {"hello": "Hello, Foo!"}}).to_string());
}

#[actix_rt::test]
async fn test_count() {
    let app = test::init_service(
        App::new()
            .app_data(actix_web::web::Data::new(
                Schema::build(CountQueryRoot, CountMutation, EmptySubscription)
                    .data(Count::default())
                    .finish(),
            ))
            .service(
                web::resource("/")
                    .guard(guard::Post())
                    .to(gql_handle_schema::<CountQueryRoot, CountMutation, EmptySubscription>),
            ),
    )
    .await;
    count_action_helper(0, r#"{"query":"{ count }"}"#, &app).await;
    count_action_helper(10, r#"{"query":"mutation{ addCount(count: 10) }"}"#, &app).await;
    count_action_helper(
        8,
        r#"{"query":"mutation{ subtractCount(count: 2) }"}"#,
        &app,
    )
    .await;
    count_action_helper(
        6,
        r#"{"query":"mutation{ subtractCount(count: 2) }"}"#,
        &app,
    )
    .await;
}

async fn count_action_helper(
    expected: i32,
    body: &'static str,
    app: &impl Service<actix_http::Request, Response = ServiceResponse, Error = actix_web::Error>,
) -> () {
    let response = test::TestRequest::post()
        .uri("/")
        .append_header((header::CONTENT_TYPE, "application/json"))
        .append_header(("Name", "Foo"))
        .set_payload(body)
        .send_request(&app)
        .await;

    assert!(response.status().is_success());
    let body = test::read_body(response).await;
    assert!(std::str::from_utf8(&body)
        .unwrap()
        .contains(&expected.to_string()));
}
