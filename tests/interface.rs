use async_graphql::*;

#[async_std::test]
pub async fn test_interface_simple_object() {
    #[derive(SimpleObject)]
    struct MyObj {
        id: i32,
        title: String,
    }

    #[derive(Interface)]
    #[graphql(field(name = "id", type = "&i32"))]
    enum Node {
        MyObj(MyObj),
    }

    struct Query;

    #[Object]
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
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        serde_json::json!({
            "node": {
                "id": 33,
            }
        })
    );
}

#[async_std::test]
pub async fn test_interface_simple_object2() {
    #[derive(SimpleObject)]
    struct MyObj {
        id: i32,
        title: String,
    }

    #[derive(Interface)]
    #[graphql(field(name = "id", type = "&i32"))]
    enum Node {
        MyObj(MyObj),
    }

    struct Query;

    #[Object]
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
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
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

    #[Object]
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

    #[derive(Interface)]
    #[graphql(field(name = "value_a", type = "i32"))]
    enum InterfaceA {
        MyObj(MyObj),
    }

    #[derive(Interface)]
    #[graphql(field(name = "value_b", type = "i32"))]
    enum InterfaceB {
        MyObj(MyObj),
    }

    struct Query;

    #[Object]
    impl Query {
        async fn my_obj(&self) -> InterfaceB {
            MyObj.into()
        }
    }

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
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
        schema.execute(query).await.into_result().unwrap().data,
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

    #[Object]
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

    #[Object]
    impl MyObjTwo {
        async fn value_a(&self) -> i32 {
            1
        }
    }

    #[derive(Interface)]
    #[graphql(field(name = "value_a", type = "i32"))]
    enum InterfaceA {
        MyObjOne(MyObjOne),
        MyObjTwo(MyObjTwo),
    }

    #[derive(Interface)]
    #[graphql(field(name = "value_b", type = "i32"))]
    enum InterfaceB {
        MyObjOne(MyObjOne),
    }

    struct Query;

    #[Object]
    impl Query {
        async fn my_obj(&self) -> Vec<InterfaceA> {
            vec![MyObjOne.into(), MyObjTwo.into()]
        }
    }

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
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
        schema.execute(query).await.into_result().unwrap().data,
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

    #[Object]
    impl MyObj {
        async fn value(&self) -> Result<i32> {
            Ok(10)
        }
    }

    #[derive(Interface)]
    #[graphql(field(name = "value", type = "Result<i32>"))]
    enum Node {
        MyObj(MyObj),
    }

    struct Query;

    #[Object]
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
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        serde_json::json!({
            "node": {
                "value": 10,
            }
        })
    );
}

#[async_std::test]
pub async fn test_interface_field_method() {
    struct A;

    #[Object]
    impl A {
        #[field(name = "created_at")]
        pub async fn created_at(&self) -> i32 {
            1
        }
    }

    struct B;

    #[Object]
    impl B {
        #[field(name = "created_at")]
        pub async fn created_at(&self) -> i32 {
            2
        }
    }

    #[derive(Interface)]
    #[graphql(field(name = "created_at", method = "created_at", type = "i32"))]
    enum MyInterface {
        A(A),
        B(B),
    }

    struct Query;

    #[Object]
    impl Query {
        async fn test(&self) -> MyInterface {
            A.into()
        }
    }

    let query = "{ test { created_at } }";
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        serde_json::json!({
            "test": {
                "created_at": 1,
            }
        })
    );
}
