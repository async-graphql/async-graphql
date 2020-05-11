use async_graphql::prelude::*;
use async_graphql::{EmptyMutation, EmptySubscription};

#[async_std::test]
pub async fn test_input_object_default_value() {
    #[GqlInputObject]
    struct MyInput {
        #[field(default = "999")]
        a: i32,

        #[field(default = "[1, 2, 3]")]
        b: Vec<i32>,

        #[field(default = "\"abc\"")]
        c: String,

        #[field(default = "999")]
        d: Option<i32>,

        #[field(default = "999")]
        e: Option<i32>,
    }

    struct MyOutput {
        a: i32,
        b: Vec<i32>,
        c: String,
        d: Option<i32>,
        e: Option<i32>,
    }

    #[GqlObject]
    impl MyOutput {
        async fn a(&self) -> i32 {
            self.a
        }

        async fn b(&self) -> &Vec<i32> {
            &self.b
        }

        async fn c(&self) -> &String {
            &self.c
        }

        async fn d(&self) -> &Option<i32> {
            &self.d
        }

        async fn e(&self) -> &Option<i32> {
            &self.e
        }
    }

    struct Root;

    #[GqlObject]
    impl Root {
        async fn a(&self, input: MyInput) -> MyOutput {
            MyOutput {
                a: input.a,
                b: input.b,
                c: input.c,
                d: input.d,
                e: input.e,
            }
        }
    }

    let schema = GqlSchema::new(Root, EmptyMutation, EmptySubscription);
    let query = format!(
        r#"{{
            a(input:{{e:777}}) {{
                a b c d e
            }}
        }}"#
    );
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "a": {
                "a": 999,
                "b": [1, 2, 3],
                "c": "abc",
                "d": 999,
                "e": 777,
            }
        })
    );
}

#[async_std::test]
pub async fn test_inputobject_derive_and_item_attributes() {
    use serde_derive::Deserialize;

    #[GqlInputObject]
    #[derive(Deserialize, PartialEq, Debug)]
    struct MyInputObject {
        #[field]
        #[serde(alias = "other")]
        real: i32,
    }

    assert_eq!(
        serde_json::from_str::<MyInputObject>(r#"{ "other" : 100 }"#).unwrap(),
        MyInputObject { real: 100 }
    );
}
