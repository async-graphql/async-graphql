use async_graphql::*;

#[async_std::test]
pub async fn test_optional_type() {
    #[InputObject]
    struct MyInput {
        value: Option<i32>,
    }

    struct Root {
        value1: Option<i32>,
        value2: Option<i32>,
    }

    #[Object]
    impl Root {
        #[field]
        async fn value1(&self) -> Option<i32> {
            self.value1.clone()
        }

        #[field]
        async fn value1_ref(&self) -> &Option<i32> {
            &self.value1
        }

        #[field]
        async fn value2(&self) -> Option<i32> {
            self.value2.clone()
        }

        #[field]
        async fn value2_ref(&self) -> &Option<i32> {
            &self.value2
        }

        #[field]
        async fn test_arg(&self, input: Option<i32>) -> Option<i32> {
            input
        }

        #[field]
        async fn test_input<'a>(&self, input: MyInput) -> Option<i32> {
            input.value.clone()
        }
    }

    let schema = Schema::new(
        Root {
            value1: Some(10),
            value2: None,
        },
        EmptyMutation,
        EmptySubscription,
    );
    let query = format!(
        r#"{{
            value1
            value1Ref
            value2
            value2Ref
            testArg1: testArg(input: 10)
            testArg2: testArg
            testInput1: testInput(input: {{value: 10}})
            testInput2: testInput(input: {{}})
            }}"#
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&schema.query(&query).execute().await.unwrap())
            .unwrap(),
        serde_json::json!({
            "value1": 10,
            "value1Ref": 10,
            "value2": null,
            "value2Ref": null,
            "testArg1": 10,
            "testArg2": null,
            "testInput1": 10,
            "testInput2": null,
        })
    );
}
