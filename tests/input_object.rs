use async_graphql::*;

#[async_std::test]
pub async fn test_input_object_default_value() {
    #[InputObject]
    struct MyInput {
        #[field(default = 999)]
        a: i32,

        #[field(default_with = "vec![1, 2, 3]")]
        b: Vec<i32>,

        #[field(default = "abc")]
        c: String,

        #[field(default = 999)]
        d: i32,

        #[field(default = 999)]
        e: i32,
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

    #[Object]
    impl Root {
        async fn a(&self, input: MyInput) -> MyOutput {
            MyOutput {
                a: input.a,
                b: input.b,
                c: input.c,
                d: Some(input.d),
                e: Some(input.e),
            }
        }
    }

    let schema = Schema::new(Root, EmptyMutation, EmptySubscription);
    let query = r#"{
            a(input:{e:777}) {
                a b c d e
            }
        }"#
    .to_owned();
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
    use serde::Deserialize;

    #[InputObject]
    #[derive(Deserialize, PartialEq, Debug)]
    struct MyInputObject {
        #[serde(alias = "other")]
        real: i32,
    }

    assert_eq!(
        serde_json::from_str::<MyInputObject>(r#"{ "other" : 100 }"#).unwrap(),
        MyInputObject { real: 100 }
    );
}

#[async_std::test]
pub async fn test_inputobject_flatten_recursive() {
    #[derive(GQLInputObject, Debug, Eq, PartialEq)]
    struct A {
        a: i32,
    }

    #[derive(GQLInputObject, Debug, Eq, PartialEq)]
    struct B {
        #[field(default = 70)]
        b: i32,
        #[field(flatten)]
        a_obj: A,
    }

    #[derive(GQLInputObject, Debug, Eq, PartialEq)]
    struct MyInputObject {
        #[field(flatten)]
        b_obj: B,
        c: i32,
    }

    assert_eq!(
        MyInputObject::parse(Some(
            Value::from_json(serde_json::json!({
               "a": 10,
               "b": 20,
               "c": 30,
            }))
            .unwrap()
        ))
        .unwrap(),
        MyInputObject {
            b_obj: B {
                b: 20,
                a_obj: A { a: 10 }
            },
            c: 30,
        }
    );

    assert_eq!(
        MyInputObject {
            b_obj: B {
                b: 20,
                a_obj: A { a: 10 }
            },
            c: 30,
        }
        .to_value(),
        Value::from_json(serde_json::json!({
           "a": 10,
           "b": 20,
           "c": 30,
        }))
        .unwrap()
    );

    struct Query;

    #[Object]
    impl Query {
        async fn test(&self, input: MyInputObject) -> i32 {
            input.c + input.b_obj.b + input.b_obj.a_obj.a
        }

        async fn test_with_default(
            &self,
            #[arg(default_with = r#"MyInputObject {
            b_obj: B {
                b: 2,
                a_obj: A { a: 1 }
            },
            c: 3,
        }"#)]
            input: MyInputObject,
        ) -> i32 {
            input.c + input.b_obj.b + input.b_obj.a_obj.a
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema
            .execute(
                r#"{
            test(input:{a:10, b: 20, c: 30})
        }"#
            )
            .await
            .unwrap()
            .data,
        serde_json::json!({
            "test": 60,
        })
    );

    assert_eq!(
        schema
            .execute(
                r#"{
            test(input:{a:10, c: 30})
        }"#
            )
            .await
            .unwrap()
            .data,
        serde_json::json!({
            "test": 110,
        })
    );

    assert_eq!(
        schema
            .execute(
                r#"{
            testWithDefault
        }"#
            )
            .await
            .unwrap()
            .data,
        serde_json::json!({
            "testWithDefault": 6,
        })
    );
}

#[async_std::test]
pub async fn test_inputobject_flatten_multiple() {
    #[derive(GQLInputObject, Debug, Eq, PartialEq)]
    struct A {
        a: i32,
    }

    #[derive(GQLInputObject, Debug, Eq, PartialEq)]
    struct B {
        b: i32,
    }

    #[derive(GQLInputObject, Debug, Eq, PartialEq)]
    struct C {
        c: i32,
    }

    #[derive(GQLInputObject, Debug, Eq, PartialEq)]
    struct ABC {
        #[field(flatten)]
        a: A,

        #[field(flatten)]
        b: B,

        #[field(flatten)]
        c: C,
    }

    assert_eq!(
        ABC::parse(Some(
            Value::from_json(serde_json::json!({
               "a": 10,
               "b": 20,
               "c": 30,
            }))
            .unwrap()
        ))
        .unwrap(),
        ABC {
            a: A { a: 10 },
            b: B { b: 20 },
            c: C { c: 30 }
        }
    );

    assert_eq!(
        ABC {
            a: A { a: 10 },
            b: B { b: 20 },
            c: C { c: 30 }
        }
        .to_value(),
        Value::from_json(serde_json::json!({
           "a": 10,
           "b": 20,
           "c": 30,
        }))
        .unwrap()
    );
}
