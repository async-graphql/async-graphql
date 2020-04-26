mod test_utils;
use async_std::prelude::*;
use async_std::task;
use serde_json::json;
use std::time::Duration;
use tide::Request;

use async_graphql::*;

#[test]
fn quickstart() -> tide::Result<()> {
    task::block_on(async {
        let port = test_utils::find_port().await;
        let server = task::spawn(async move {
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
            app.listen(&port).await?;

            Ok(())
        });

        let client = task::spawn(async move {
            task::sleep(Duration::from_millis(100)).await;
            let string = surf::post(format!("http://{}", port))
                .body_bytes(r#"{"query":"{ add(a: 10, b: 20) }"}"#)
                .set_header("Content-Type".parse().unwrap(), "application/json")
                .recv_string()
                .await?;
            assert_eq!(string, json!({"data": {"add": 30}}).to_string());
            Ok(())
        });

        server.race(client).await
    })
}

#[test]
fn hello() -> tide::Result<()> {
    task::block_on(async {
        let port = test_utils::find_port().await;
        let server = task::spawn(async move {
            struct Hello(String);
            struct QueryRoot;
            #[Object]
            impl QueryRoot {
                #[field(desc = "Returns hello")]
                async fn hello<'a>(&self, ctx: &'a Context<'_>) -> String {
                    let name = ctx.data_opt::<Hello>().map(|hello| hello.0.as_str());
                    format!(
                        "Hello, {}!",
                        if let Some(name) = name { name } else { "world" }
                    )
                }
            }

            struct ServerState {
                schema: Schema<QueryRoot, EmptyMutation, EmptySubscription>,
            }
            let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

            let server_state = ServerState { schema };
            let mut app = tide::with_state(server_state);

            app.at("/").post(|req: Request<ServerState>| async move {
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
            app.listen(&port).await?;

            Ok(())
        });

        let client = task::spawn(async move {
            task::sleep(Duration::from_millis(100)).await;
            let string = surf::post(format!("http://{}", port))
                .body_bytes(r#"{"query":"{ hello }"}"#)
                .set_header("Content-Type".parse().unwrap(), "application/json")
                .set_header("Name".parse().unwrap(), "Foo")
                .recv_string()
                .await?;
            assert_eq!(string, json!({"data":{"hello":"Hello, Foo!"}}).to_string());

            let string = surf::post(format!("http://{}", port))
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

        server.race(client).await
    })
}
