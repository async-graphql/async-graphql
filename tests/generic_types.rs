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
        serde_json::json!({
            "objI32": {"value": 100},
            "objBool": {"value": true},
        })
    );
}
