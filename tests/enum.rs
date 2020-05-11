use async_graphql::prelude::*;
use async_graphql::{EmptyMutation, EmptySubscription};

#[async_std::test]
pub async fn test_enum_type() {
    #[GqlEnum]
    enum MyEnum {
        A,
        B,
    }

    #[GqlInputObject]
    struct MyInput {
        value: MyEnum,
    }

    struct Root {
        value: MyEnum,
    }

    #[GqlObject]
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

    let schema = GqlSchema::new(Root { value: MyEnum::A }, EmptyMutation, EmptySubscription);
    let query = format!(
        r#"{{
            value
            testArg(input: A)
            testInput(input: {{value: B}}) }}
            "#
    );
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "value": "A",
            "testArg": "A",
            "testInput": "B",
        })
    );
}

#[async_std::test]
pub async fn test_enum_derive_and_item_attributes() {
    use serde_derive::Deserialize;

    #[GqlEnum]
    #[derive(Deserialize, PartialEq, Debug)]
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
