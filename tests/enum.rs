use async_graphql::*;

#[tokio::test]
pub async fn test_enum_type() {
    #[derive(Enum, Copy, Clone, Eq, PartialEq)]
    enum MyEnum {
        A,
        B,
    }

    #[derive(InputObject)]
    struct MyInput {
        value: MyEnum,
    }

    struct Root {
        value: MyEnum,
    }

    #[Object]
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
        value!({
            "value": "A",
            "testArg": "A",
            "testInput": "B",
        })
    );
}

#[tokio::test]
pub async fn test_enum_derive_and_item_attributes() {
    use serde::Deserialize;

    #[derive(Deserialize, Debug, Enum, Copy, Clone, Eq, PartialEq)]
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
        from_value::<TestStruct>(value!({"value": "Other"})).unwrap(),
        TestStruct { value: Test::Real }
    );
}

#[tokio::test]
pub async fn test_remote_enum() {
    #[derive(Enum, Copy, Clone, Eq, PartialEq)]
    #[graphql(remote = "remote::RemoteEnum")]
    enum LocalEnum {
        A,
        B,
        C,
    }

    mod remote {
        pub enum RemoteEnum {
            A,
            B,
            C,
        }
    }

    let _: remote::RemoteEnum = LocalEnum::A.into();
    let _: LocalEnum = remote::RemoteEnum::A.into();
}

#[tokio::test]
pub async fn test_display() {
    #[derive(Enum, Copy, Clone, Eq, PartialEq)]
    #[graphql(display)]
    enum MyEnum {
        A,
        #[graphql(name = "bbb")]
        B,
        C,
    }

    assert_eq!(MyEnum::A.to_string(), "A");
    assert_eq!(MyEnum::B.to_string(), "bbb");
    assert_eq!(MyEnum::C.to_string(), "C");
}
