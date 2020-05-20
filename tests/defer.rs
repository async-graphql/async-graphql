use async_graphql::*;
use futures::StreamExt;

#[async_std::test]
pub async fn test_defer() {
    #[derive(Clone)]
    struct MyObj;

    #[Object]
    impl MyObj {
        async fn value(&self) -> i32 {
            20
        }

        async fn obj(&self) -> Deferred<MyObj> {
            MyObj.into()
        }
    }

    struct Query;

    #[Object]
    impl Query {
        async fn value(&self) -> Deferred<i32> {
            10.into()
        }

        async fn obj(&self) -> Deferred<MyObj> {
            MyObj.into()
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = r#"{
        value @defer
    }"#;
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "value": 10,
        })
    );

    let query = r#"{
        value @defer
        obj @defer {
            value
            obj @defer {
                value
            }
        }
    }"#;
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "value": 10,
            "obj": {
                "value": 20,
                "obj": {
                    "value": 20
                }
            }
        })
    );

    let mut stream = schema.execute_stream(&query);
    assert_eq!(
        stream.next().await.unwrap().unwrap().data,
        serde_json::json!({
            "value": null,
            "obj": null,
        })
    );

    let next_resp = stream.next().await.unwrap().unwrap();
    assert_eq!(next_resp.path, Some(vec![serde_json::json!("value")]));
    assert_eq!(next_resp.data, serde_json::json!(10));

    let next_resp = stream.next().await.unwrap().unwrap();
    assert_eq!(next_resp.path, Some(vec![serde_json::json!("obj")]));
    assert_eq!(
        next_resp.data,
        serde_json::json!({"value": 20, "obj": null})
    );

    let next_resp = stream.next().await.unwrap().unwrap();
    assert_eq!(
        next_resp.path,
        Some(vec![serde_json::json!("obj"), serde_json::json!("obj")])
    );
    assert_eq!(next_resp.data, serde_json::json!({"value": 20}));

    assert!(stream.next().await.is_none());
}
