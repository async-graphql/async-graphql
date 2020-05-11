use async_graphql::prelude::*;
use async_graphql::{EmptyMutation, EmptySubscription, Pos, QueryError};

#[async_std::test]
pub async fn test_complexity_and_depth() {
    struct Query;

    struct MyObj;

    #[GqlObject]
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

    #[GqlObject]
    impl Query {
        async fn value(&self) -> i32 {
            1
        }

        async fn obj(&self) -> MyObj {
            MyObj
        }
    }

    let query = "{ a:value b:value c:value }";
    let schema = GqlSchema::build(Query, EmptyMutation, EmptySubscription)
        .limit_complexity(2)
        .finish();
    assert_eq!(
        schema.execute(&query).await.unwrap_err(),
        GqlError::Query {
            pos: Pos { line: 0, column: 0 },
            path: None,
            err: QueryError::TooComplex,
        }
    );

    let query = "{ a:value b:value }";
    let schema = GqlSchema::build(Query, EmptyMutation, EmptySubscription)
        .limit_complexity(2)
        .finish();
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "a": 1,
            "b": 1,
        })
    );

    let query = "{ obj { a b } }";
    let schema = GqlSchema::build(Query, EmptyMutation, EmptySubscription)
        .limit_complexity(2)
        .finish();
    assert_eq!(
        schema.execute(&query).await.unwrap_err(),
        GqlError::Query {
            pos: Pos { line: 0, column: 0 },
            path: None,
            err: QueryError::TooComplex,
        }
    );

    let query = "{ obj { a } }";
    let schema = GqlSchema::build(Query, EmptyMutation, EmptySubscription)
        .limit_complexity(2)
        .finish();
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
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
    let schema = GqlSchema::build(Query, EmptyMutation, EmptySubscription)
        .limit_depth(2)
        .finish();
    assert_eq!(
        schema.execute(&query).await.unwrap_err(),
        GqlError::Query {
            pos: Pos { line: 0, column: 0 },
            path: None,
            err: QueryError::TooDeep,
        }
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
    let schema = GqlSchema::build(Query, EmptyMutation, EmptySubscription)
        .limit_depth(3)
        .finish();
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
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
