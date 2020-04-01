use async_graphql::*;

#[async_std::test]
pub async fn test_input_object_default_value() {
    #[InputObject]
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

    #[Object]
    impl MyOutput {
        #[field]
        async fn a(&self) -> i32 {
            self.a
        }

        #[field]
        async fn b(&self) -> &Vec<i32> {
            &self.b
        }

        #[field]
        async fn c(&self) -> &String {
            &self.c
        }

        #[field]
        async fn d(&self) -> &Option<i32> {
            &self.d
        }

        #[field]
        async fn e(&self) -> &Option<i32> {
            &self.e
        }
    }

    struct Root;

    #[Object]
    impl Root {
        #[field]
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

    let schema = Schema::new(Root, EmptyMutation, EmptySubscription);
    let query = format!(
        r#"{{
            a(input:{{e:777}}) {{
                a b c d e
            }}
        }}"#
    );
    assert_eq!(
        schema.query(&query).unwrap().execute().await.unwrap().data,
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
