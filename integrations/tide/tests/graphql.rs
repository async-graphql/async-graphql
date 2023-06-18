#![allow(clippy::uninlined_format_args)]

mod test_utils;

use std::io::Read;

use async_graphql::*;
use reqwest::{header, StatusCode};
use serde_json::json;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[async_std::test]
async fn quickstart() -> Result<()> {
    let listen_addr = "127.0.0.1:8081";

    async_std::task::spawn(async move {
        struct QueryRoot;
        #[Object]
        impl QueryRoot {
            /// Returns the sum of a and b
            async fn add(&self, a: i32, b: i32) -> i32 {
                a + b
            }
        }

        let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

        let mut app = tide::new();
        let endpoint = async_graphql_tide::graphql(schema);
        app.at("/").post(endpoint.clone()).get(endpoint);
        app.listen(listen_addr).await
    });

    test_utils::wait_server_ready().await;

    let client = test_utils::client();

    let resp = client
        .post(&format!("http://{}", listen_addr))
        .json(&json!({"query":"{ add(a: 10, b: 20) }"}))
        .send()
        .await?;

    assert_eq!(resp.status(), StatusCode::OK);
    let string = resp.text().await?;
    println!("via post {}", string);

    assert_eq!(string, json!({"data": {"add": 30}}).to_string());

    let resp = client
        .get(&format!("http://{}", listen_addr))
        .query(&[("query", "{ add(a: 10, b: 20) }")])
        .send()
        .await?;

    assert_eq!(resp.status(), StatusCode::OK);
    let string = resp.text().await?;
    println!("via get {}", string);

    assert_eq!(string, json!({"data": {"add": 30}}).to_string());

    Ok(())
}

#[async_std::test]
async fn hello() -> Result<()> {
    let listen_addr = "127.0.0.1:8082";

    async_std::task::spawn(async move {
        struct Hello(String);
        struct QueryRoot;
        #[Object]
        impl QueryRoot {
            /// Returns hello
            async fn hello<'a>(&self, ctx: &'a Context<'_>) -> String {
                let name = ctx.data_opt::<Hello>().map(|hello| hello.0.as_str());
                format!("Hello, {}!", name.unwrap_or("world"))
            }
        }

        let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

        let mut app = tide::new();

        app.at("/").post(move |req: tide::Request<()>| {
            let schema = schema.clone();
            async move {
                let name = req
                    .header("name")
                    .and_then(|values| values.get(0))
                    .map(ToString::to_string);
                let mut req = async_graphql_tide::receive_request(req).await?;
                if let Some(name) = name {
                    req = req.data(Hello(name));
                }
                async_graphql_tide::respond(schema.execute(req).await)
            }
        });
        app.listen(listen_addr).await
    });

    test_utils::wait_server_ready().await;

    let client = test_utils::client();

    let resp = client
        .post(&format!("http://{}", listen_addr))
        .json(&json!({"query":"{ hello }"}))
        .header("Name", "Foo")
        .send()
        .await?;

    assert_eq!(resp.status(), StatusCode::OK);
    let string = resp.text().await?;
    println!("{}", string);

    assert_eq!(string, json!({"data":{"hello":"Hello, Foo!"}}).to_string());

    let resp = client
        .post(&format!("http://{}", listen_addr))
        .json(&json!({"query":"{ hello }"}))
        .header(header::CONTENT_TYPE, "application/json")
        .send()
        .await?;

    assert_eq!(resp.status(), StatusCode::OK);
    let string = resp.text().await?;
    println!("{}", string);

    assert_eq!(
        string,
        json!({"data":{"hello":"Hello, world!"}}).to_string()
    );

    Ok(())
}

#[async_std::test]
async fn upload() -> Result<()> {
    let listen_addr = "127.0.0.1:8083";

    async_std::task::spawn(async move {
        struct QueryRoot;

        #[Object]
        impl QueryRoot {
            async fn value(&self) -> i32 {
                10
            }
        }

        #[derive(Clone, SimpleObject)]
        pub struct FileInfo {
            filename: String,
            mime_type: Option<String>,
        }

        struct MutationRoot;
        #[Object]
        impl MutationRoot {
            async fn single_upload(&self, ctx: &Context<'_>, file: Upload) -> FileInfo {
                let upload_value = file.value(ctx).unwrap();
                println!("single_upload: filename={}", upload_value.filename);
                println!(
                    "single_upload: content_type={:?}",
                    upload_value.content_type
                );

                let file_info = FileInfo {
                    filename: upload_value.filename.clone(),
                    mime_type: upload_value.content_type.clone(),
                };

                let mut content = String::new();
                upload_value
                    .into_read()
                    .read_to_string(&mut content)
                    .unwrap();
                assert_eq!(content, "test".to_owned());

                file_info
            }
        }

        let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription).finish();

        let mut app = tide::new();
        app.at("/").post(async_graphql_tide::graphql(schema));
        app.listen(listen_addr).await
    });

    test_utils::wait_server_ready().await;

    let client = test_utils::client();

    let form = reqwest::multipart::Form::new()
        .text("operations", r#"{ "query": "mutation ($file: Upload!) { singleUpload(file: $file) { filename, mimeType } }", "variables": { "file": null } }"#)
        .text("map", r#"{ "0": ["variables.file"] }"#)
        .part("0", reqwest::multipart::Part::stream("test").file_name("test.txt").mime_str("text/plain")?);

    let resp = client
        .post(&format!("http://{}", listen_addr))
        .multipart(form)
        .send()
        .await?;

    assert_eq!(resp.status(), StatusCode::OK);
    let string = resp.text().await?;
    println!("{}", string);

    assert_eq!(
        string,
        json!({"data": {"singleUpload": {"filename": "test.txt", "mimeType": "text/plain"}}})
            .to_string()
    );

    Ok(())
}
