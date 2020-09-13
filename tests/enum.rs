use async_graphql::*;

#[async_std::test]
pub async fn test_enum_type() {
    #[derive(GQLEnum, Copy, Clone, Eq, PartialEq)]
    enum MyEnum {
        A,
        B,
    }

    #[derive(GQLInputObject)]
    struct MyInput {
        value: MyEnum,
    }

    struct Root {
        value: MyEnum,
    }

    #[GQLObject]
    impl Root {
        async fn value(&self) -> MyEnum {
            self.value
        }

        async fn test_arg(&self, input: MyEnum) -> MyEnum {
            input
        }

        async fn test_input<'a>(&self, input: MyInput) -> MyEnum {
            input.value
        }
    }

    let schema = Schema::new(Root { value: MyEnum::A }, EmptyMutation, EmptySubscription);
    let query = r#"{
            value
            testArg(input: A)
            testInput(input: {value: B})
        }"#
    .to_owned();
    assert_eq!(
        schema.execute(&query).await.data,
        serde_json::json!({
            "value": "A",
            "testArg": "A",
            "testInput": "B",
        })
    );
}

#[async_std::test]
pub async fn test_enum_derive_and_item_attributes() {
    use serde::Deserialize;

    #[derive(Deserialize, Debug, GQLEnum, Copy, Clone, Eq, PartialEq)]
    enum Test {
        #[serde(alias = "Other")]
        Real,
    }

    #[derive(Deserialize, PartialEq, Debug)]
    #[allow(dead_code)]
    struct TestStruct {
        value: Test,
    }

    assert_eq!(
        serde_json::from_str::<TestStruct>(r#"{ "value" : "Other" }"#).unwrap(),
        TestStruct { value: Test::Real }
    );
}
