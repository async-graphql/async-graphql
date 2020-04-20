use async_graphql::*;

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

    #[async_graphql::Interface(implements = "InterfaceA", field(name = "value_b", type = "i32"))]
    struct InterfaceB(MyObj);

    struct Query;

    #[Object]
    impl Query {
        #[field]
        async fn my_obj(&self) -> InterfaceB {
            MyObj.into()
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
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
