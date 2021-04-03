mod test_utils;
use actix_http::Request;
use actix_web::dev::{MessageBody, Service, ServiceResponse};
use actix_web::{guard, test, web, App};
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

    let req = test::TestRequest::with_uri("/").to_request();

    let resp = app.call(req).await.unwrap();
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    assert!(std::str::from_utf8(&body).unwrap().contains("graphql"));
}

#[actix_rt::test]
async fn test_add() {
    let app = test::init_service(
        App::new()
            .data(Schema::new(AddQueryRoot, EmptyMutation, EmptySubscription))
            .service(
                web::resource("/")
                    .guard(guard::Post())
                    .to(gql_handle_schema::<AddQueryRoot, EmptyMutation, EmptySubscription>),
            ),
    )
    .await;

    let resp = test::TestRequest::post()
        .uri("/")
        .set_payload(r#"{"query":"{ add(a: 10, b: 20) }"}"#)
        .send_request(&app)
        .await;

    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    assert_eq!(body, json!({"data": {"add": 30}}).to_string());
}

#[actix_rt::test]
async fn test_hello() {
    let app = test::init_service(
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
            ),
    )
    .await;

    let resp = test::TestRequest::post()
        .uri("/")
        .set_payload(r#"{"query":"{ hello }"}"#)
        .send_request(&app)
        .await;

    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    assert_eq!(
        body,
        json!({"data": {"hello": "Hello, world!"}}).to_string()
    );
}

#[actix_rt::test]
async fn test_hello_header() {
    let app = test::init_service(
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
            ),
    )
    .await;

    let resp = test::TestRequest::post()
        .uri("/")
        .append_header(("Name", "Foo"))
        .set_payload(r#"{"query":"{ hello }"}"#)
        .send_request(&app)
        .await;

    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    assert_eq!(body, json!({"data": {"hello": "Hello, Foo!"}}).to_string());
}

#[actix_rt::test]
async fn test_count() {
    let app = test::init_service(
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

async fn count_action_helper<S, B, E>(expected: i32, payload: &'static str, app: &S)
where
    S: Service<Request, Response = ServiceResponse<B>, Error = E>,
    B: MessageBody + Unpin,
    E: std::fmt::Debug,
{
    let resp = test::TestRequest::post()
        .uri("/")
        .set_payload(payload)
        .send_request(app)
        .await;

    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    assert!(std::str::from_utf8(&body)
        .unwrap()
        .contains(&expected.to_string()));
}
