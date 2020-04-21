use async_graphql::*;

#[async_std::test]
pub async fn test_interface_simple_object() {
    #[async_graphql::SimpleObject]
    pub struct MyObj {
        #[field]
        pub id: i32,
        #[field]
        pub title: String,
    }

    #[async_graphql::Interface(field(name = "id", type = "i32"))]
    pub struct Node(MyObj);

    struct Query;

    #[Object]
    impl Query {
        #[field]
        async fn node(&self) -> Node {
            MyObj {
                id: 33,
                title: "haha".to_string(),
            }
            .into()
        }
    }

    let query = format!(
        r#"{{
            node {{
                ... on Node {{
                    id
                }}
            }}
        }}"#
    );
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
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
    #[async_graphql::SimpleObject]
    pub struct MyObj {
        #[field(ref)]
        pub id: i32,
        #[field]
        pub title: String,
    }

    #[async_graphql::Interface(field(name = "id", type = "&i32"))]
    pub struct Node(MyObj);

    struct Query;

    #[Object]
    impl Query {
        #[field]
        async fn node(&self) -> Node {
            MyObj {
                id: 33,
                title: "haha".to_string(),
            }
            .into()
        }
    }

    let query = format!(
        r#"{{
            node {{
                ... on Node {{
                    id
                }}
            }}
        }}"#
    );
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
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

    #[async_graphql::Object]
    impl MyObj {
        #[field]
        async fn value_a(&self) -> i32 {
            1
        }

        #[field]
        async fn value_b(&self) -> i32 {
            2
        }

        #[field]
        async fn value_c(&self) -> i32 {
            3
        }
    }

    #[async_graphql::Interface(field(name = "value_a", type = "i32"))]
    struct InterfaceA(MyObj);

    #[async_graphql::Interface(field(name = "value_b", type = "i32"))]
    struct InterfaceB(MyObj);

    struct Query;

    #[Object]
    impl Query {
        #[field]
        async fn my_obj(&self) -> InterfaceB {
            MyObj.into()
        }
    }

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .register_type::<InterfaceA>() // `InterfaceA` is not directly referenced, so manual registration is required.
        .finish();
    let query = format!(
        r#"{{
            myObj {{
               ... on InterfaceA {{
                valueA
              }}
              ... on InterfaceB {{
                valueB
              }}
              ... on MyObj {{
                valueC
              }}
            }}
        }}"#
    );
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
