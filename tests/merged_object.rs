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
            MyObj(Object1 { a: 10 }, Object2 { b: 20 }, Object3 { c: 30 })
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

#[async_std::test]
pub async fn test_merged_object_derive() {
    #[derive(GQLMergedObject)]
    struct MyObj(Object1, Object2, Object3);

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self) -> MyObj {
            MyObj(Object1 { a: 10 }, Object2 { b: 20 }, Object3 { c: 30 })
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

#[async_std::test]
pub async fn test_merged_object_default() {
    mod a {
        use super::*;

        #[derive(GQLSimpleObject)]
        pub struct QueryA {
            pub a: i32,
        }

        impl Default for QueryA {
            fn default() -> Self {
                Self { a: 10 }
            }
        }
    }

    mod b {
        use super::*;

        #[derive(GQLSimpleObject)]
        pub struct QueryB {
            pub b: i32,
        }

        impl Default for QueryB {
            fn default() -> Self {
                Self { b: 20 }
            }
        }
    }

    #[derive(GQLMergedObject, Default)]
    struct Query(a::QueryA, b::QueryB);

    let schema = Schema::new(Query::default(), EmptyMutation, EmptySubscription);
    let query = "{ a b }";
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "a": 10,
            "b": 20,
        })
    );
}
