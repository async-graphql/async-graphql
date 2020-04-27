mod test_utils;
use async_std::prelude::*;
use async_std::task;
use serde_json::json;
use std::time::Duration;
use tide::Request;

use async_graphql::*;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[test]
fn quickstart() -> Result<()> {
    task::block_on(async {
        let listen_addr = test_utils::find_listen_addr().await;

        let server: task::JoinHandle<Result<()>> = task::spawn(async move {
            struct QueryRoot;
            #[Object]
            impl QueryRoot {
                #[field(desc = "Returns the sum of a and b")]
                async fn add(&self, a: i32, b: i32) -> i32 {
                    a + b
                }
            }

            let mut app = tide::new();
            app.at("/").post(|req: Request<()>| async move {
                let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();
                async_graphql_tide::graphql(req, schema, |query_builder| query_builder).await
            });
            app.listen(&listen_addr).await?;

            Ok(())
        });

        let client: task::JoinHandle<Result<()>> = task::spawn(async move {
            task::sleep(Duration::from_millis(300)).await;

            let string = surf::post(format!("http://{}", listen_addr))
                .body_bytes(r#"{"query":"{ add(a: 10, b: 20) }"}"#)
                .set_header("Content-Type".parse().unwrap(), "application/json")
                .recv_string()
                .await?;

            assert_eq!(string, json!({"data": {"add": 30}}).to_string());

            Ok(())
        });

        server.race(client).await?;

        Ok(())
    })
}

#[test]
fn hello() -> Result<()> {
    task::block_on(async {
        let listen_addr = test_utils::find_listen_addr().await;

        let server: task::JoinHandle<Result<()>> = task::spawn(async move {
            struct Hello(String);
            struct QueryRoot;
            #[Object]
            impl QueryRoot {
                #[field(desc = "Returns hello")]
                async fn hello<'a>(&self, ctx: &'a Context<'_>) -> String {
                    let name = ctx.data_opt::<Hello>().map(|hello| hello.0.as_str());
                    format!("Hello, {}!", name.unwrap_or("world"))
                }
            }

            struct AppState {
                schema: Schema<QueryRoot, EmptyMutation, EmptySubscription>,
            }
            let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

            let app_state = AppState { schema };
            let mut app = tide::with_state(app_state);

            app.at("/").post(|req: Request<AppState>| async move {
                let schema = req.state().schema.clone();
                let name = &req
                    .header(&"name".parse().unwrap())
                    .and_then(|values| values.first().map(|value| value.to_string()));

                async_graphql_tide::graphql(req, schema, |mut query_builder| {
                    if let Some(name) = name {
                        query_builder = query_builder.data(Hello(name.to_string()))
                    }
                    query_builder
                })
                .await
            });
            app.listen(&listen_addr).await?;

            Ok(())
        });

        let client: task::JoinHandle<Result<()>> = task::spawn(async move {
            task::sleep(Duration::from_millis(300)).await;

            let string = surf::post(format!("http://{}", listen_addr))
                .body_bytes(r#"{"query":"{ hello }"}"#)
                .set_header("Content-Type".parse().unwrap(), "application/json")
                .set_header("Name".parse().unwrap(), "Foo")
                .recv_string()
                .await?;

            assert_eq!(string, json!({"data":{"hello":"Hello, Foo!"}}).to_string());

            let string = surf::post(format!("http://{}", listen_addr))
                .body_bytes(r#"{"query":"{ hello }"}"#)
                .set_header("Content-Type".parse().unwrap(), "application/json")
                .recv_string()
                .await?;

            assert_eq!(
                string,
                json!({"data":{"hello":"Hello, world!"}}).to_string()
            );

            Ok(())
        });

        server.race(client).await?;

        Ok(())
    })
}
