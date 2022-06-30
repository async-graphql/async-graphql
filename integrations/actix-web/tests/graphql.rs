use actix_http::Method;
use actix_web::{dev::Service, guard, test, web, web::Data, App};
use async_graphql::*;
use serde_json::json;
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
    let body = response.into_body();
    assert!(
        std::str::from_utf8(&actix_web::body::to_bytes(body).await.unwrap())
            .unwrap()
            .contains("graphql")
    );
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
    let body = response.into_body();
    assert_eq!(
        actix_web::body::to_bytes(body).await.unwrap(),
        json!({"data": {"add": 30}}).to_string().into_bytes()
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
    let body = response.into_body();
    assert_eq!(
        actix_web::body::to_bytes(body).await.unwrap(),
        json!({"data": {"hello": "Hello, world!"}})
            .to_string()
            .into_bytes()
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
    let body = response.into_body();
    assert_eq!(
        actix_web::body::to_bytes(body).await.unwrap(),
        json!({"data": {"hello": "Hello, Foo!"}})
            .to_string()
            .into_bytes()
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
    let body = response.into_body();
    assert_eq!(
        actix_web::body::to_bytes(body).await.unwrap(),
        json!({"data": {"count": 0}}).to_string().into_bytes()
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
    let body = response.into_body();
    assert_eq!(
        actix_web::body::to_bytes(body).await.unwrap(),
        json!({"data": {"addCount": 10}}).to_string().into_bytes(),
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
    let body = response.into_body();
    assert_eq!(
        actix_web::body::to_bytes(body).await.unwrap(),
        json!({"data": {"subtractCount": 8}})
            .to_string()
            .into_bytes()
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
    let body = response.into_body();
    assert_eq!(
        actix_web::body::to_bytes(body).await.unwrap(),
        json!({"data": {"subtractCount": 6}})
            .to_string()
            .into_bytes()
    );
}

#[cfg(feature = "cbor")]
#[actix_rt::test]
async fn test_cbor() {
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
                .insert_header((actix_http::header::ACCEPT, "application/cbor"))
                .to_request(),
        )
        .await
        .unwrap();
    assert!(response.status().is_success());
    #[derive(Debug, serde::Deserialize, PartialEq)]
    struct Response {
        data: ResponseInner,
    }
    #[derive(Debug, serde::Deserialize, PartialEq)]
    struct ResponseInner {
        add: i32,
    }
    let body = actix_web::body::to_bytes(response.into_body())
        .await
        .unwrap();
    let response: Response = serde_cbor::from_slice(&body).unwrap();
    assert_eq!(
        response,
        Response {
            data: ResponseInner { add: 30 }
        }
    );
}
