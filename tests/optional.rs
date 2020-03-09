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
        GQLEmptyMutation,
    );
    let query = format!(
        r#"{{
            value1
            value1_ref
            value2
            value2_ref
            test_arg1: test_arg(input: 10)
            test_arg2: test_arg
            test_input1: test_input(input: {{value: 10}})
            test_input2: test_input(input: {{}})
            }}"#
    );
    assert_eq!(
        schema.query(&query).execute().await.unwrap(),
        serde_json::json!({
            "value1": 10,
            "value1_ref": 10,
            "value2": null,
            "value2_ref": null,
            "test_arg1": 10,
            "test_arg2": null,
            "test_input1": 10,
            "test_input2": null,
        })
    );
}
