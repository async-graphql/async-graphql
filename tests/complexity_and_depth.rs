use async_graphql::*;

#[async_std::test]
pub async fn test_complexity_and_depth() {
    struct Query;

    struct MyObj;

    #[Object]
    impl MyObj {
        async fn a(&self) -> i32 {
            1
        }

        async fn b(&self) -> i32 {
            2
        }

        async fn c(&self) -> MyObj {
            MyObj
        }
    }

    #[Object]
    impl Query {
        async fn value(&self) -> i32 {
            1
        }

        async fn obj(&self) -> MyObj {
            MyObj
        }
    }

    let query = "{ a:value b:value c:value }";
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .limit_complexity(2)
        .finish();
    assert_eq!(
        schema.execute(query).await.into_result().unwrap_err(),
        vec![ServerError {
            message: "Query is too complex.".to_owned(),
            locations: Vec::new(),
            path: Vec::new(),
            extensions: None,
        }]
    );

    let query = "{ a:value b:value }";
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .limit_complexity(2)
        .finish();
    assert_eq!(
        schema.execute(query).await.data,
        serde_json::json!({
            "a": 1,
            "b": 1,
        })
    );

    let query = "{ obj { a b } }";
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .limit_complexity(2)
        .finish();
    assert_eq!(
        schema.execute(query).await.into_result().unwrap_err(),
        vec![ServerError {
            message: "Query is too complex.".to_owned(),
            locations: Vec::new(),
            path: Vec::new(),
            extensions: None,
        }]
    );

    let query = "{ obj { a } }";
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .limit_complexity(2)
        .finish();
    assert_eq!(
        schema.execute(query).await.data,
        serde_json::json!({
            "obj": { "a": 1 }
        })
    );

    let query = r#"
        {
            obj { # 0
                a b c { # 1
                    a b c { # 2
                        a b # 3
                    }
                }
            }
        }"#;
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .limit_depth(2)
        .finish();
    assert_eq!(
        schema.execute(query).await.into_result().unwrap_err(),
        vec![ServerError {
            message: "Query is nested too deep.".to_owned(),
            locations: Vec::new(),
            path: Vec::new(),
            extensions: None,
        }]
    );

    let query = r#"
        {
            obj { # 0
                a b c { # 1
                    a b c { # 2
                        a b # 3
                    }
                }
            }
        }"#;
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .limit_depth(3)
        .finish();
    assert_eq!(
        schema.execute(query).await.data,
        serde_json::json!({
            "obj": {
                "a": 1,
                "b": 2,
                "c": {
                    "a": 1,
                    "b": 2,
                    "c": {
                        "a": 1,
                        "b": 2,
                    }
                }
            }
        })
    );
}
