use async_graphql::prelude::*;
use async_graphql::{EmptyMutation, EmptySubscription};

#[async_std::test]
pub async fn test_union_simple_object() {
    #[GqlSimpleObject]
    struct MyObj {
        id: i32,
        title: String,
    }

    #[GqlUnion]
    enum Node {
        MyObj(MyObj),
    }

    struct Query;

    #[GqlObject]
    impl Query {
        async fn node(&self) -> Node {
            MyObj {
                id: 33,
                title: "haha".to_string(),
            }
            .into()
        }
    }

    let query = r#"{
            node {
                ... on MyObj {
                    id
                }
            }
        }"#;
    let schema = GqlSchema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "node": {
                "id": 33,
            }
        })
    );
}

#[async_std::test]
pub async fn test_union_simple_object2() {
    #[GqlSimpleObject]
    struct MyObj {
        #[field(ref)]
        id: i32,
        title: String,
    }

    #[GqlUnion]
    enum Node {
        MyObj(MyObj),
    }

    struct Query;

    #[GqlObject]
    impl Query {
        async fn node(&self) -> Node {
            MyObj {
                id: 33,
                title: "haha".to_string(),
            }
            .into()
        }
    }

    let query = r#"{
            node {
                ... on MyObj {
                    id
                }
            }
        }"#;
    let schema = GqlSchema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "node": {
                "id": 33,
            }
        })
    );
}

#[async_std::test]
pub async fn test_multiple_unions() {
    struct MyObj;

    #[GqlObject]
    impl MyObj {
        async fn value_a(&self) -> i32 {
            1
        }

        async fn value_b(&self) -> i32 {
            2
        }

        async fn value_c(&self) -> i32 {
            3
        }
    }

    #[GqlUnion]
    enum UnionA {
        MyObj(MyObj),
    }

    #[GqlUnion]
    enum UnionB {
        MyObj(MyObj),
    }

    struct Query;

    #[GqlObject]
    impl Query {
        async fn union_a(&self) -> UnionA {
            MyObj.into()
        }
        async fn union_b(&self) -> UnionB {
            MyObj.into()
        }
    }

    let schema = GqlSchema::build(Query, EmptyMutation, EmptySubscription)
        .register_type::<UnionA>() // `UnionA` is not directly referenced, so manual registration is required.
        .finish();
    let query = r#"{
            unionA {
               ... on MyObj {
                valueA
                valueB
                valueC
              }
            }
            unionB {
                ... on MyObj {
                 valueA
                 valueB
                 valueC
               }
             }
        }"#;
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "unionA": {
                "valueA": 1,
                "valueB": 2,
                "valueC": 3,
            },
            "unionB": {
                "valueA": 1,
                "valueB": 2,
                "valueC": 3,
            }
        })
    );
}

#[async_std::test]
pub async fn test_multiple_objects_in_multiple_unions() {
    struct MyObjOne;

    #[GqlObject]
    impl MyObjOne {
        async fn value_a(&self) -> i32 {
            1
        }

        async fn value_b(&self) -> i32 {
            2
        }

        async fn value_c(&self) -> i32 {
            3
        }
    }

    struct MyObjTwo;

    #[GqlObject]
    impl MyObjTwo {
        async fn value_a(&self) -> i32 {
            1
        }
    }

    #[GqlUnion]
    enum UnionA {
        MyObjOne(MyObjOne),
        MyObjTwo(MyObjTwo),
    }

    #[GqlUnion]
    enum UnionB {
        MyObjOne(MyObjOne),
    }

    struct Query;

    #[GqlObject]
    impl Query {
        async fn my_obj(&self) -> Vec<UnionA> {
            vec![MyObjOne.into(), MyObjTwo.into()]
        }
    }

    let schema = GqlSchema::build(Query, EmptyMutation, EmptySubscription)
        .register_type::<UnionB>() // `UnionB` is not directly referenced, so manual registration is required.
        .finish();
    let query = r#"{
            myObj {
                ... on MyObjTwo {
                    valueA
                }
                ... on MyObjOne {
                    valueA
                    valueB
                    valueC
                }
            }
         }"#;
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "myObj": [{
                "valueA": 1,
                "valueB": 2,
                "valueC": 3,
            }, {
                "valueA": 1
            }]
        })
    );
}

#[async_std::test]
pub async fn test_union_field_result() {
    struct MyObj;

    #[GqlObject]
    impl MyObj {
        async fn value(&self) -> GqlFieldResult<i32> {
            Ok(10)
        }
    }

    #[GqlUnion]
    enum Node {
        MyObj(MyObj),
    }

    struct Query;

    #[GqlObject]
    impl Query {
        async fn node(&self) -> Node {
            MyObj.into()
        }
    }

    let query = r#"{
            node {
                ... on MyObj {
                    value
                }
            }
        }"#;
    let schema = GqlSchema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "node": {
                "value": 10,
            }
        })
    );
}
