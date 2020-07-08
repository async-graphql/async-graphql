use async_graphql::*;
use std::collections::HashMap;

#[async_std::test]
pub async fn test_json_scalar() {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct MyData(HashMap<String, i32>);

    #[derive(serde::Serialize, Clone)]
    struct MyDataOutput(HashMap<String, i32>);

    struct Query {
        data: MyDataOutput,
    }

    #[Object]
    impl Query {
        async fn data(&self) -> Json<MyData> {
            let mut items = HashMap::new();
            items.insert("a".to_string(), 10);
            items.insert("b".to_string(), 20);
            Json(MyData(items))
        }

        async fn data_output(&self) -> OutputJson<&MyDataOutput> {
            OutputJson(&self.data)
        }

        async fn data_output_clone(&self) -> OutputJson<MyDataOutput> {
            OutputJson(self.data.clone())
        }
    }

    let schema = Schema::new(
        Query {
            data: {
                let mut items = HashMap::new();
                items.insert("a".to_string(), 10);
                items.insert("b".to_string(), 20);
                MyDataOutput(items)
            },
        },
        EmptyMutation,
        EmptySubscription,
    );
    let query = r#"{ data dataOutput dataOutputClone }"#;
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "data": { "a": 10, "b": 20},
            "dataOutput": { "a": 10, "b": 20},
            "dataOutputClone": { "a": 10, "b": 20},
        })
    );
}
