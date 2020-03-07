use async_graphql::*;

#[async_std::test]
pub async fn test_list_type() {
    #[InputObject]
    struct MyInput {
        value: Vec<i32>,
    }

    struct Root {
        value: Vec<i32>,
    }

    #[Object]
    impl Root {
        #[field]
        async fn value_vec(&self) -> Vec<i32> {
            self.value.clone()
        }

        #[field]
        async fn value_slice(&self) -> &[i32] {
            &self.value
        }

        #[field]
        async fn test_arg(&self, input: Vec<i32>) -> Vec<i32> {
            input
        }

        #[field]
        async fn test_input<'a>(&self, input: MyInput) -> Vec<i32> {
            input.value
        }
    }

    let schema = Schema::new(
        Root {
            value: vec![1, 2, 3, 4, 5],
        },
        GQLEmptyMutation,
    );
    let json_value: serde_json::Value = vec![1, 2, 3, 4, 5].into();
    let query = format!(
        r#"{{
            value_vec
            value_slice
            test_arg(input: {0})
            test_input(input: {{value: {0}}}) }}
            "#,
        json_value
    );
    assert_eq!(
        schema.query(&query).execute().await.unwrap(),
        serde_json::json!({
            "value_vec": vec![1, 2, 3, 4, 5],
            "value_slice": vec![1, 2, 3, 4, 5],
            "test_arg": vec![1, 2, 3, 4, 5],
            "test_input": vec![1, 2, 3, 4, 5],
        })
    );
}
