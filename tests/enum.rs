use async_graphql::*;

#[async_std::test]
pub async fn test_enum_type() {
    #[Enum]
    enum MyEnum {
        A,
        B,
    }

    #[InputObject]
    struct MyInput {
        value: MyEnum,
    }

    struct Root {
        value: MyEnum,
    }

    #[Object]
    impl Root {
        #[field]
        async fn value(&self) -> MyEnum {
            self.value
        }

        #[field]
        async fn test_arg(&self, input: MyEnum) -> MyEnum {
            input
        }

        #[field]
        async fn test_input<'a>(&self, input: MyInput) -> MyEnum {
            input.value
        }
    }

    let schema = Schema::new(Root { value: MyEnum::A }, EmptyMutation, EmptySubscription);
    let query = format!(
        r#"{{
            value
            testArg(input: A)
            testInput(input: {{value: B}}) }}
            "#
    );
    assert_eq!(
        schema.query(&query).unwrap().execute().await.unwrap().data,
        serde_json::json!({
            "value": "A",
            "testArg": "A",
            "testInput": "B",
        })
    );
}
