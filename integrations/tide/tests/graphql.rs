mod test_utils;
use serde_json::json;
use smol::{Task, Timer};
use std::io::Read;
use std::time::Duration;

use async_graphql::*;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[test]
fn quickstart() -> Result<()> {
    smol::run(async {
        let listen_addr = test_utils::find_listen_addr().await;

        let server = Task::<Result<()>>::spawn(async move {
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
            let endpoint = async_graphql_tide::endpoint(schema);
            app.at("/").post(endpoint.clone()).get(endpoint);
            app.listen(listen_addr).await?;

            Ok(())
        });

        let client = Task::<Result<()>>::spawn(async move {
            Timer::after(Duration::from_millis(300)).await;

            let resp = reqwest::Client::builder()
                .no_proxy()
                .build()
                .unwrap()
                .post(format!("http://{}", listen_addr).as_str())
                .body(r#"{"query":"{ add(a: 10, b: 20) }"}"#)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .await?;

            assert_eq!(resp.status(), reqwest::StatusCode::OK);
            let string = resp.text().await?;
            println!("via post {}", string);

            assert_eq!(string, json!({"data": {"add": 30}}).to_string());

            //
            let resp = reqwest::Client::builder()
                .no_proxy()
                .build()
                .unwrap()
                .get(format!("http://{}", listen_addr).as_str())
                .query(&[("query", "{ add(a: 10, b: 20) }")])
                .send()
                .await?;

            assert_eq!(resp.status(), reqwest::StatusCode::OK);
            let string = resp.text().await?;
            println!("via get {}", string);

            assert_eq!(string, json!({"data": {"add": 30}}).to_string());

            Ok(())
        });

        client.await?;
        server.cancel().await;

        Ok(())
    })
}

#[test]
fn hello() -> Result<()> {
    smol::run(async {
        let listen_addr = test_utils::find_listen_addr().await;

        let server = Task::<Result<()>>::spawn(async move {
            use tide::Request;

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

            app.at("/").post(move |req: Request<()>| {
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
            app.listen(listen_addr).await?;

            Ok(())
        });

        let client = Task::<Result<()>>::spawn(async move {
            Timer::after(Duration::from_millis(300)).await;

            let resp = reqwest::Client::builder()
                .no_proxy()
                .build()
                .unwrap()
                .post(format!("http://{}", listen_addr).as_str())
                .body(r#"{"query":"{ hello }"}"#)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .header("Name", "Foo")
                .send()
                .await?;

            assert_eq!(resp.status(), reqwest::StatusCode::OK);
            let string = resp.text().await?;
            println!("{}", string);

            assert_eq!(string, json!({"data":{"hello":"Hello, Foo!"}}).to_string());

            let resp = reqwest::Client::builder()
                .no_proxy()
                .build()
                .unwrap()
                .post(format!("http://{}", listen_addr).as_str())
                .body(r#"{"query":"{ hello }"}"#)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .await?;

            assert_eq!(resp.status(), reqwest::StatusCode::OK);
            let string = resp.text().await?;
            println!("{}", string);

            assert_eq!(
                string,
                json!({"data":{"hello":"Hello, world!"}}).to_string()
            );

            Ok(())
        });

        client.await?;
        server.cancel().await;

        Ok(())
    })
}

#[test]
fn upload() -> Result<()> {
    smol::run(async {
        let listen_addr = test_utils::find_listen_addr().await;

        let server = Task::<Result<()>>::spawn(async move {
            struct QueryRoot;
            #[Object]
            impl QueryRoot {}

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
            app.at("/").post(async_graphql_tide::endpoint(schema));
            app.listen(listen_addr).await?;

            Ok(())
        });

        let client = Task::<Result<()>>::spawn(async move {
            Timer::after(Duration::from_millis(300)).await;

            let form = reqwest::multipart::Form::new()
                .text("operations", r#"{ "query": "mutation ($file: Upload!) { singleUpload(file: $file) { filename, mimeType } }", "variables": { "file": null } }"#)
                .text("map", r#"{ "0": ["variables.file"] }"#)
                .part("0", reqwest::multipart::Part::stream("test").file_name("test.txt").mime_str("text/plain")?);

            let resp = reqwest::Client::builder()
                .no_proxy()
                .build()
                .unwrap()
                .post(format!("http://{}", listen_addr).as_str())
                .multipart(form)
                .send()
                .await?;

            assert_eq!(resp.status(), reqwest::StatusCode::OK);
            let string = resp.text().await?;
            println!("{}", string);

            assert_eq!(
                string,
                json!({"data": {"singleUpload": {"filename": "test.txt", "mimeType": "text/plain"}}}).to_string()
            );

            Ok(())
        });

        client.await?;
        server.cancel().await;

        Ok(())
    })
}
