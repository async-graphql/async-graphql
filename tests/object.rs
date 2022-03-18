use async_graphql::*;

#[tokio::test]
async fn test_flatten() {
    #[derive(SimpleObject)]
    struct A {
        a: i32,
        b: i32,
    }

    struct B;

    #[Object]
    impl B {
        #[graphql(flatten)]
        async fn a(&self) -> A {
            A { a: 100, b: 200 }
        }

        async fn c(&self) -> i32 {
            300
        }
    }

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self) -> B {
            B
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = "{ __type(name: \"B\") { fields { name } } }";
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "__type": {
                "fields": [
                    {"name": "a"},
                    {"name": "b"},
                    {"name": "c"}
                ]
            }
        })
    );

    let query = "{ obj { a b c } }";
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "obj": {
                "a": 100,
                "b": 200,
                "c": 300,
            }
        })
    );
}

#[tokio::test]
async fn test_flatten_with_context() {
    #[derive(SimpleObject)]
    struct A {
        a: i32,
        b: i32,
    }

    struct B;

    #[Object]
    impl B {
        #[graphql(flatten)]
        async fn a(&self, _ctx: &Context<'_>) -> A {
            A { a: 100, b: 200 }
        }

        async fn c(&self) -> i32 {
            300
        }
    }

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self) -> B {
            B
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = "{ __type(name: \"B\") { fields { name } } }";
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "__type": {
                "fields": [
                    {"name": "a"},
                    {"name": "b"},
                    {"name": "c"}
                ]
            }
        })
    );

    let query = "{ obj { a b c } }";
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "obj": {
                "a": 100,
                "b": 200,
                "c": 300,
            }
        })
    );
}

#[tokio::test]
async fn test_oneof_field() {
    #[derive(OneofObject)]
    enum TestArg {
        A(i32),
        B(String),
    }

    struct Query;

    #[Object]
    impl Query {
        #[graphql(oneof)]
        async fn test(&self, arg: TestArg) -> String {
            match arg {
                TestArg::A(a) => format!("a:{}", a),
                TestArg::B(b) => format!("b:{}", b),
            }
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = "{ test(a: 10) }";
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "test": "a:10"
        })
    );

    let query = r#"{ test(b: "abc") }"#;
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "test": "b:abc"
        })
    );

    let query = r#"{
        __type(name: "Query") {
            fields {
                name
                args {
                    name
                    type {
                        kind
                        name
                    }
                }
            }
        }
    }"#;
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "__type": {
                "fields": [{
                    "name": "test",
                    "args": [{
                        "name": "a",
                        "type": {
                            "kind": "SCALAR",
                            "name": "Int"
                        }
                    }, {
                        "name": "b",
                        "type": {
                            "kind": "SCALAR",
                            "name": "String"
                        }
                    }]
                }]
            }
        })
    );
}
