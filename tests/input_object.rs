use async_graphql::*;

#[tokio::test]
pub async fn test_input_object_default_value() {
    #[derive(InputObject)]
    struct MyInput {
        #[graphql(default = 999)]
        a: i32,

        #[graphql(default_with = "vec![1, 2, 3]")]
        b: Vec<i32>,

        #[graphql(default = "abc")]
        c: String,

        #[graphql(default = 999)]
        d: i32,

        #[graphql(default = 999)]
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
        schema.execute(&query).await.data,
        value!({
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

#[tokio::test]
pub async fn test_inputobject_derive_and_item_attributes() {
    use serde::Deserialize;

    #[derive(Deserialize, PartialEq, Debug, InputObject)]
    struct MyInputObject {
        #[serde(alias = "other")]
        real: i32,
    }

    assert_eq!(
        serde_json::from_str::<MyInputObject>(r#"{ "other" : 100 }"#).unwrap(),
        MyInputObject { real: 100 }
    );
}

#[tokio::test]
pub async fn test_inputobject_flatten_recursive() {
    #[derive(InputObject, Debug, Eq, PartialEq)]
    struct A {
        a: i32,
    }

    #[derive(InputObject, Debug, Eq, PartialEq)]
    struct B {
        #[graphql(default = 70)]
        b: i32,
        #[graphql(flatten)]
        a_obj: A,
    }

    #[derive(InputObject, Debug, Eq, PartialEq)]
    struct MyInputObject {
        #[graphql(flatten)]
        b_obj: B,
        c: i32,
    }

    assert_eq!(
        MyInputObject::parse(Some(value!({
           "a": 10,
           "b": 20,
           "c": 30,
        })))
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
        value!({
           "a": 10,
           "b": 20,
           "c": 30,
        })
    );

    struct Query;

    #[Object]
    impl Query {
        async fn test(&self, input: MyInputObject) -> i32 {
            input.c + input.b_obj.b + input.b_obj.a_obj.a
        }

        async fn test_with_default(
            &self,
            #[graphql(default_with = r#"MyInputObject {
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
            .into_result()
            .unwrap()
            .data,
        value!({
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
            .into_result()
            .unwrap()
            .data,
        value!({
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
            .into_result()
            .unwrap()
            .data,
        value!({
            "testWithDefault": 6,
        })
    );
}

#[tokio::test]
pub async fn test_inputobject_flatten_multiple() {
    #[derive(InputObject, Debug, Eq, PartialEq)]
    struct A {
        a: i32,
    }

    #[derive(InputObject, Debug, Eq, PartialEq)]
    struct B {
        b: i32,
    }

    #[derive(InputObject, Debug, Eq, PartialEq)]
    struct C {
        c: i32,
    }

    #[derive(InputObject, Debug, Eq, PartialEq)]
    struct Abc {
        #[graphql(flatten)]
        a: A,

        #[graphql(flatten)]
        b: B,

        #[graphql(flatten)]
        c: C,
    }

    assert_eq!(
        Abc::parse(Some(value!({
           "a": 10,
           "b": 20,
           "c": 30,
        })))
        .unwrap(),
        Abc {
            a: A { a: 10 },
            b: B { b: 20 },
            c: C { c: 30 }
        }
    );

    assert_eq!(
        Abc {
            a: A { a: 10 },
            b: B { b: 20 },
            c: C { c: 30 }
        }
        .to_value(),
        value!({
           "a": 10,
           "b": 20,
           "c": 30,
        })
    );
}

#[tokio::test]
pub async fn test_input_object_skip_field() {
    #[derive(InputObject)]
    struct MyInput2 {
        a: i32,
        #[graphql(skip)]
        b: i32,
    }

    struct Root;

    #[Object]
    impl Root {
        async fn a(&self, input: MyInput2) -> i32 {
            assert_eq!(input.b, i32::default());
            input.a
        }
    }

    let schema = Schema::new(Root, EmptyMutation, EmptySubscription);
    let query = r#"{
            a(input:{a: 777})
        }"#;
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "a": 777
        })
    );
}

#[tokio::test]
pub async fn test_box_input_object() {
    #[derive(InputObject)]
    struct MyInput {
        value: i32,
        input: Option<Box<MyInput>>,
    }

    struct Root;

    #[Object]
    impl Root {
        async fn q(&self, input: MyInput) -> i32 {
            input.value
                + input.input.as_ref().unwrap().value
                + input.input.as_ref().unwrap().input.as_ref().unwrap().value
        }
    }

    let schema = Schema::new(Root, EmptyMutation, EmptySubscription);
    let query = r#"{
            q(input: {value: 100, input: { value: 200, input: { value: 300 } } })
        }"#;
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "q": 600
        })
    );
}
