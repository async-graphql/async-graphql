use async_graphql::*;

#[SimpleObject]
struct Object1 {
    a: i32,
}

#[SimpleObject]
struct Object2 {
    b: i32,
}

#[SimpleObject]
struct Object3 {
    c: i32,
}

#[async_std::test]
pub async fn test_merged_object() {
    type MyObj =
        MergedObject<Object1, MergedObject<Object2, MergedObject<Object3, MergedObjectTail>>>;

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self) -> MyObj {
            MergedObject(
                Object1 { a: 10 },
                MergedObject(
                    Object2 { b: 20 },
                    MergedObject(Object3 { c: 30 }, MergedObjectTail),
                ),
            )
        }
    }

    assert_eq!(
        MyObj::type_name(),
        "Object1_Object2_Object3_MergedObjectTail"
    );

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = "{ obj { a b c } }";
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "obj": {
                "a": 10,
                "b": 20,
                "c": 30,
            }
        })
    );
}

#[async_std::test]
pub async fn test_merged_object_macro() {
    #[MergedObject]
    struct MyObj(Object1, Object2, Object3);

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self) -> MyObj {
            MyObj::new(Object1 { a: 10 }, Object2 { b: 20 }, Object3 { c: 30 })
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = "{ obj { a b c } }";
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "obj": {
                "a": 10,
                "b": 20,
                "c": 30,
            }
        })
    );
}
