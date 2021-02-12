use async_graphql::*;

#[async_std::test]
pub async fn test_generic_object() {
    struct MyObj<T> {
        value: T,
    }

    #[Object(name = "MyObjI32")]
    impl MyObj<i32> {
        async fn value(&self) -> i32 {
            self.value
        }
    }

    #[Object(name = "MyObjBool")]
    impl MyObj<bool> {
        async fn value(&self) -> bool {
            self.value
        }
    }

    struct Query;

    #[Object]
    impl Query {
        async fn obj_i32(&self) -> MyObj<i32> {
            MyObj { value: 100 }
        }

        async fn obj_bool(&self) -> MyObj<bool> {
            MyObj { value: true }
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = r#"{
            objI32 { value }
            objBool { value }
        }"#
    .to_owned();
    assert_eq!(
        schema.execute(&query).await.into_result().unwrap().data,
        value!({
            "objI32": {"value": 100},
            "objBool": {"value": true},
        })
    );
}

#[async_std::test]
pub async fn test_input_object_generic() {
    #[derive(InputObject)]
    #[graphql(
        concrete(name = "IntEqualityFilter", params(i32)),
        concrete(name = "StringEqualityFilter", params(String))
    )]
    struct EqualityFilter<T: InputType> {
        equals: Option<T>,
        not_equals: Option<T>,
    }

    assert_eq!(EqualityFilter::<i32>::type_name(), "IntEqualityFilter");
    assert_eq!(
        EqualityFilter::<String>::type_name(),
        "StringEqualityFilter"
    );

    struct Query;

    #[Object]
    impl Query {
        async fn q1(&self, input: EqualityFilter<i32>) -> i32 {
            input.equals.unwrap_or_default() + input.not_equals.unwrap_or_default()
        }

        async fn q2(&self, input: EqualityFilter<String>) -> String {
            input.equals.unwrap_or_default() + &input.not_equals.unwrap_or_default()
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = r#"{
            q1(input: { equals: 7, notEquals: 8 } )
            q2(input: { equals: "ab", notEquals: "cd" } )
        }"#;
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "q1": 15,
            "q2": "abcd",
        })
    );

    assert_eq!(
        schema
            .execute(
                r#"{ __type(name: "IntEqualityFilter") { inputFields { name type { name } } } }"#
            )
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "__type": {
                "inputFields": [
                    {"name": "equals", "type": { "name": "Int" } },
                    {"name": "notEquals", "type": { "name": "Int" } },
                ]
            }
        })
    );

    assert_eq!(
        schema
            .execute(r#"{ __type(name: "Query") { fields { name args { name type { kind ofType { name } } } } } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "__type": {
                "fields": [
                    {
                        "name": "q1", 
                        "args": [{
                            "name": "input",
                            "type": {
                                "kind": "NON_NULL",
                                "ofType": { "name": "IntEqualityFilter" },
                            },
                        }]
                    },
                    {
                        "name": "q2", 
                        "args": [{
                            "name": "input",
                            "type": {
                                "kind": "NON_NULL",
                                "ofType": { "name": "StringEqualityFilter" },
                            },
                        }],
                    }
                ]
            }
        })
    );
}

#[async_std::test]
pub async fn test_generic_simple_object() {
    #[derive(SimpleObject)]
    #[graphql(concrete(name = "MyObjIntString", params(i32, String)))]
    #[graphql(concrete(name = "MyObji64f32", params(i64, u8)))]
    struct MyObj<A: OutputType, B: OutputType> {
        a: A,
        b: B,
    }

    struct Query;

    #[Object]
    impl Query {
        async fn q1(&self) -> MyObj<i32, String> {
            MyObj {
                a: 100,
                b: "abc".to_string(),
            }
        }

        async fn q2(&self) -> MyObj<i64, u8> {
            MyObj { a: 100, b: 28 }
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = "{ q1 { a b } q2 { a b } }";
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "q1": {
                "a": 100,
                "b": "abc",
            },
            "q2": {
                "a": 100,
                "b": 28,
            }
        })
    );

    assert_eq!(
        schema
            .execute(r#"{ __type(name: "MyObjIntString") { fields { name type { kind ofType { name } } } } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "__type": {
                "fields": [
                    {
                        "name": "a",
                        "type": {
                            "kind": "NON_NULL",
                            "ofType": { "name": "Int" },
                        },
                    },
                    {
                        "name": "b",
                        "type": {
                            "kind": "NON_NULL",
                            "ofType": { "name": "String" },
                        },
                    },
                ]
            }
        })
    );

    assert_eq!(
        schema
            .execute(
                r#"{ __type(name: "Query") { fields { name type { kind ofType { name } } } } }"#
            )
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "__type": {
                "fields": [
                    {
                        "name": "q1",
                        "type": {
                            "kind": "NON_NULL",
                            "ofType": { "name": "MyObjIntString" },
                        },
                    },
                    {
                        "name": "q2",
                        "type": {
                            "kind": "NON_NULL",
                            "ofType": { "name": "MyObji64f32" },
                        },
                    },
                ]
            }
        })
    );
}
