use async_graphql::prelude::*;
use async_graphql::{EmptyMutation, EmptySubscription};

#[async_std::test]
pub async fn test_interface_simple_object() {
    #[GqlSimpleObject]
    struct MyObj {
        id: i32,
        title: String,
    }

    #[GqlInterface(field(name = "id", type = "i32"))]
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
                ... on Node {
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
pub async fn test_interface_simple_object2() {
    #[GqlSimpleObject]
    struct MyObj {
        #[field(ref)]
        id: i32,
        title: String,
    }

    #[GqlInterface(field(name = "id", type = "&i32"))]
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
                ... on Node {
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
pub async fn test_multiple_interfaces() {
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

    #[GqlInterface(field(name = "value_a", type = "i32"))]
    enum InterfaceA {
        MyObj(MyObj),
    }

    #[GqlInterface(field(name = "value_b", type = "i32"))]
    enum InterfaceB {
        MyObj(MyObj),
    }

    struct Query;

    #[GqlObject]
    impl Query {
        async fn my_obj(&self) -> InterfaceB {
            MyObj.into()
        }
    }

    let schema = GqlSchema::build(Query, EmptyMutation, EmptySubscription)
        .register_type::<InterfaceA>() // `InterfaceA` is not directly referenced, so manual registration is required.
        .finish();
    let query = r#"{
            myObj {
               ... on InterfaceA {
                valueA
              }
              ... on InterfaceB {
                valueB
              }
              ... on MyObj {
                valueC
              }
            }
        }"#;
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "myObj": {
                "valueA": 1,
                "valueB": 2,
                "valueC": 3,
            }
        })
    );
}

#[async_std::test]
pub async fn test_multiple_objects_in_multiple_interfaces() {
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

    #[GqlInterface(field(name = "value_a", type = "i32"))]
    enum InterfaceA {
        MyObjOne(MyObjOne),
        MyObjTwo(MyObjTwo),
    }

    #[GqlInterface(field(name = "value_b", type = "i32"))]
    enum InterfaceB {
        MyObjOne(MyObjOne),
    }

    struct Query;

    #[GqlObject]
    impl Query {
        async fn my_obj(&self) -> Vec<InterfaceA> {
            vec![MyObjOne.into(), MyObjTwo.into()]
        }
    }

    let schema = GqlSchema::build(Query, EmptyMutation, EmptySubscription)
        .register_type::<InterfaceB>() // `InterfaceB` is not directly referenced, so manual registration is required.
        .finish();
    let query = r#"{
             myObj {
                ... on InterfaceA {
                 valueA
               }
               ... on InterfaceB {
                 valueB
               }
               ... on MyObjOne {
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
pub async fn test_interface_field_result() {
    struct MyObj;

    #[GqlObject]
    impl MyObj {
        async fn value(&self) -> GqlFieldResult<i32> {
            Ok(10)
        }
    }

    #[GqlInterface(field(name = "value", type = "GqlFieldResult<i32>"))]
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
                ... on Node {
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
