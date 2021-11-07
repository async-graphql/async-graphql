use actix_http::Method;
use actix_web::dev::{AnyBody, Service};
use actix_web::{guard, test, web, web::Data, App};
use serde_json::json;

use async_graphql::*;
use test_utils::*;

mod test_utils;

#[actix_rt::test]
async fn test_playground() {
    let srv = test::init_service(
        App::new().service(
            web::resource("/")
                .guard(guard::Get())
                .to(test_utils::gql_playgound),
        ),
    )
    .await;
    let req = test::TestRequest::with_uri("/").to_request();
    let response = srv.call(req).await.unwrap();
    assert!(response.status().is_success());
    let body = response.response().body();
    if let AnyBody::Bytes(bytes) = body {
        assert!(std::str::from_utf8(&bytes).unwrap().contains("graphql"));
    } else {
        panic!("response body must be Bytes {:?}", body);
    }
}

#[actix_rt::test]
async fn test_add() {
    let srv = test::init_service(
        App::new()
            .app_data(Data::new(Schema::new(
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
    let response = srv
        .call(
            test::TestRequest::with_uri("/")
                .method(Method::POST)
                .set_payload(r#"{"query":"{ add(a: 10, b: 20) }"}"#)
                .to_request(),
        )
        .await
        .unwrap();
    assert!(response.status().is_success());
    let body = response.response().body();
    assert_eq!(
        body,
        &AnyBody::Bytes(json!({"data": {"add": 30}}).to_string().into_bytes().into())
    );
}

#[actix_rt::test]
async fn test_hello() {
    let srv = test::init_service(
        App::new()
            .app_data(Data::new(Schema::new(
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

    let response = srv
        .call(
            test::TestRequest::with_uri("/")
                .method(Method::POST)
                .set_payload(r#"{"query":"{ hello }"}"#)
                .to_request(),
        )
        .await
        .unwrap();
    assert!(response.status().is_success());
    let body = response.response().body();
    assert_eq!(
        body,
        &AnyBody::Bytes(
            json!({"data": {"hello": "Hello, world!"}})
                .to_string()
                .into_bytes()
                .into()
        )
    );
}

#[actix_rt::test]
async fn test_hello_header() {
    let srv = test::init_service(
        App::new()
            .app_data(Data::new(Schema::new(
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

    let response = srv
        .call(
            test::TestRequest::with_uri("/")
                .method(Method::POST)
                .insert_header(("Name", "Foo"))
                .set_payload(r#"{"query":"{ hello }"}"#)
                .to_request(),
        )
        .await
        .unwrap();
    assert!(response.status().is_success());
    let body = response.response().body();
    assert_eq!(
        body,
        &AnyBody::Bytes(
            json!({"data": {"hello": "Hello, Foo!"}})
                .to_string()
                .into_bytes()
                .into()
        )
    );
}

#[actix_rt::test]
async fn test_count() {
    let srv = test::init_service(
        App::new()
            .app_data(Data::new(
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

    let response = srv
        .call(
            test::TestRequest::with_uri("/")
                .method(Method::POST)
                .set_payload(r#"{"query":"{ count }"}"#)
                .to_request(),
        )
        .await
        .unwrap();
    assert!(response.status().is_success());
    let body = response.response().body();
    assert_eq!(
        body,
        &AnyBody::Bytes(
            json!({"data": {"count": 0}})
                .to_string()
                .into_bytes()
                .into()
        )
    );

    let response = srv
        .call(
            test::TestRequest::with_uri("/")
                .method(Method::POST)
                .set_payload(r#"{"query":"mutation{ addCount(count: 10) }"}"#)
                .to_request(),
        )
        .await
        .unwrap();
    assert!(response.status().is_success());
    let body = response.response().body();
    assert_eq!(
        body,
        &AnyBody::Bytes(
            json!({"data": {"addCount": 10}})
                .to_string()
                .into_bytes()
                .into()
        )
    );

    let response = srv
        .call(
            test::TestRequest::with_uri("/")
                .method(Method::POST)
                .set_payload(r#"{"query":"mutation{ subtractCount(count: 2) }"}"#)
                .to_request(),
        )
        .await
        .unwrap();
    assert!(response.status().is_success());
    let body = response.response().body();
    assert_eq!(
        body,
        &AnyBody::Bytes(
            json!({"data": {"subtractCount": 8}})
                .to_string()
                .into_bytes()
                .into()
        )
    );

    let response = srv
        .call(
            test::TestRequest::with_uri("/")
                .method(Method::POST)
                .set_payload(r#"{"query":"mutation{ subtractCount(count: 2) }"}"#)
                .to_request(),
        )
        .await
        .unwrap();
    assert!(response.status().is_success());
    let body = response.response().body();
    assert_eq!(
        body,
        &AnyBody::Bytes(
            json!({"data": {"subtractCount": 6}})
                .to_string()
                .into_bytes()
                .into()
        )
    );
}