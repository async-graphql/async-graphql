use async_graphql::*;

#[async_std::test]
pub async fn test_field_merge() {
    struct Query;

    #[Object]
    impl Query {
        async fn value1(&self) -> i32 {
            1
        }

        async fn value2(&self) -> i32 {
            2
        }

        async fn value3(&self) -> i32 {
            3
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = r#"
        {
            value1
            ... { value2 }
            ... A
        }

        fragment A on Query {
            value3
        }
    "#;
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "value1": 1,
            "value2": 2,
            "value3": 3,
        })
    );
}

#[async_std::test]
pub async fn test_field_object_merge() {
    #[SimpleObject]
    struct MyObject {
        a: i32,
        b: i32,
        c: i32,
    }

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self) -> MyObject {
            MyObject { a: 1, b: 2, c: 3 }
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = r#"
        {
            obj { a }
            ... { obj { b } }
            ... A
        }

        fragment A on Query {
            obj { c }
        }
    "#;
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "obj": {
                "a": 1,
                "b": 2,
                "c": 3,
            }
        })
    );
}
