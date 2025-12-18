use async_graphql::*;
use futures_util::stream::{Stream, StreamExt};

#[derive(SimpleObject)]
struct Object1 {
    a: i32,
}

#[derive(SimpleObject)]
struct Object2 {
    b: i32,
}

#[derive(SimpleObject)]
struct Object3 {
    c: i32,
}

#[tokio::test]
pub async fn test_merged_object_macro() {
    #[derive(MergedObject)]
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
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "obj": {
                "a": 10,
                "b": 20,
                "c": 30,
            }
        })
    );
}

#[tokio::test]
pub async fn test_merged_object_derive() {
    #[derive(MergedObject)]
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
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "obj": {
                "a": 10,
                "b": 20,
                "c": 30,
            }
        })
    );
}

#[tokio::test]
pub async fn test_merged_object_default() {
    mod a {
        use super::*;

        #[derive(SimpleObject)]
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

        #[derive(SimpleObject)]
        pub struct QueryB {
            pub b: i32,
        }

        impl Default for QueryB {
            fn default() -> Self {
                Self { b: 20 }
            }
        }
    }

    #[derive(MergedObject, Default)]
    struct Query(a::QueryA, b::QueryB);

    let schema = Schema::new(Query::default(), EmptyMutation, EmptySubscription);
    let query = "{ a b }";
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "a": 10,
            "b": 20,
        })
    );
}

#[tokio::test]
pub async fn test_merged_subscription() {
    #[derive(Default)]
    struct Subscription1;

    #[Subscription]
    impl Subscription1 {
        async fn events1(&self) -> impl Stream<Item = i32> {
            futures_util::stream::iter(0..10)
        }
    }

    #[derive(Default)]
    struct Subscription2;

    #[Subscription]
    impl Subscription2 {
        async fn events2(&self) -> impl Stream<Item = i32> {
            futures_util::stream::iter(10..20)
        }
    }

    #[derive(MergedSubscription, Default)]
    struct Subscription(Subscription1, Subscription2);

    struct Query;

    #[Object]
    impl Query {
        async fn value(&self) -> i32 {
            10
        }
    }

    let schema = Schema::new(Query, EmptyMutation, Subscription::default());

    {
        let mut stream = schema
            .execute_stream("subscription { events1 }")
            .map(|resp| resp.into_result().unwrap().data);
        for i in 0i32..10 {
            assert_eq!(
                value!({
                    "events1": i,
                }),
                stream.next().await.unwrap()
            );
        }
        assert!(stream.next().await.is_none());
    }

    {
        let mut stream = schema
            .execute_stream("subscription { events2 }")
            .map(|resp| resp.into_result().unwrap().data);
        for i in 10i32..20 {
            assert_eq!(
                value!({
                    "events2": i,
                }),
                stream.next().await.unwrap()
            );
        }
        assert!(stream.next().await.is_none());
    }
}

#[tokio::test]
pub async fn test_generic_merged_subscription() {
    struct Subscription1<T> {
        values: Vec<T>,
    }

    #[Subscription]
    impl<T> Subscription1<T>
    where
        T: Clone + OutputType + OutputTypeMarker,
    {
        async fn events1(&self) -> impl Stream<Item = T> {
            futures_util::stream::iter(self.values.clone())
        }
    }

    struct Subscription2<T> {
        values: Vec<T>,
    }

    #[Subscription]
    impl<T> Subscription2<T>
    where
        T: Clone + OutputType + OutputTypeMarker,
    {
        async fn events2(&self) -> impl Stream<Item = T> {
            futures_util::stream::iter(self.values.clone())
        }
    }

    #[derive(MergedSubscription)]
    struct Subscription<T1, T2>(Subscription1<T1>, Subscription2<T2>)
    where
        T1: Clone + OutputType + OutputTypeMarker,
        T2: Clone + OutputType + OutputTypeMarker;

    struct Query;

    #[Object]
    impl Query {
        async fn value(&self) -> i32 {
            10
        }
    }

    let subscription = Subscription(
        Subscription1 {
            values: vec![1, 2, 3],
        },
        Subscription2 {
            values: vec!["a", "b", "c"],
        },
    );
    let schema = Schema::new(Query, EmptyMutation, subscription);

    {
        let mut stream = schema
            .execute_stream("subscription { events1 }")
            .map(|resp| resp.into_result().unwrap().data);
        for i in &[1, 2, 3] {
            assert_eq!(
                value!({
                    "events1": i,
                }),
                stream.next().await.unwrap()
            );
        }
        assert!(stream.next().await.is_none());
    }

    {
        let mut stream = schema
            .execute_stream("subscription { events2 }")
            .map(|resp| resp.into_result().unwrap().data);
        for i in ["a", "b", "c"] {
            assert_eq!(
                value!({
                    "events2": i,
                }),
                stream.next().await.unwrap()
            );
        }
        assert!(stream.next().await.is_none());
    }
}

#[tokio::test]
pub async fn test_merged_entity() {
    #[derive(SimpleObject)]
    struct Fruit {
        id: ID,
        name: String,
    }

    #[derive(SimpleObject)]
    struct Vegetable {
        id: ID,
        name: String,
    }

    #[derive(Default)]
    struct FruitQuery;

    #[Object]
    impl FruitQuery {
        #[graphql(entity)]
        async fn get_fruit(&self, id: ID) -> Fruit {
            Fruit {
                id,
                name: "Apple".into(),
            }
        }
    }

    #[derive(Default)]
    struct VegetableQuery;

    #[Object]
    impl VegetableQuery {
        #[graphql(entity)]
        async fn get_vegetable(&self, id: ID) -> Vegetable {
            Vegetable {
                id,
                name: "Carrot".into(),
            }
        }
    }

    #[derive(MergedObject, Default)]
    struct Query(FruitQuery, VegetableQuery);

    let schema = Schema::new(Query::default(), EmptyMutation, EmptySubscription);
    let query = r#"{
            _entities(representations: [{__typename: "Fruit", id: "1"}]) {
                __typename
                ... on Fruit {
                    id
                    name
                }
            }
        }"#;
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "_entities": [
                {"__typename": "Fruit", "id": "1", "name": "Apple"},
            ]
        })
    );
}

#[tokio::test]
pub async fn test_issue_316() {
    #[derive(SimpleObject)]
    struct Fruit {
        id: ID,
        name: String,
    }

    struct Query;

    #[Object]
    impl Query {
        #[graphql(entity)]
        async fn get_fruit(&self, id: ID) -> Fruit {
            Fruit {
                id,
                name: "Apple".into(),
            }
        }
    }

    #[derive(Default)]
    struct Mutation1;

    #[Object]
    impl Mutation1 {
        async fn action1(&self) -> Fruit {
            Fruit {
                id: ID("hello".into()),
                name: "Apple".into(),
            }
        }
    }

    #[derive(MergedObject, Default)]
    struct Mutation(Mutation1);

    // This works
    let schema = Schema::new(Query, Mutation1, EmptySubscription);
    assert!(schema.execute("{ _service { sdl }}").await.is_ok());

    // This fails
    let schema = Schema::new(Query, Mutation::default(), EmptySubscription);
    assert!(schema.execute("{ _service { sdl }}").await.is_ok());
}

#[tokio::test]
pub async fn test_issue_333() {
    #[derive(SimpleObject)]
    struct ObjectA<'a> {
        field_a: &'a str,
    }

    #[derive(SimpleObject)]
    struct ObjectB<'a> {
        field_b: &'a str,
    }

    #[derive(MergedObject)]
    pub struct Object<'a>(ObjectA<'a>, ObjectB<'a>);

    struct Query {
        a: String,
        b: String,
    }

    #[Object]
    impl Query {
        async fn obj(&self) -> Object<'_> {
            Object(ObjectA { field_a: &self.a }, ObjectB { field_b: &self.b })
        }
    }

    let schema = Schema::new(
        Query {
            a: "haha".to_string(),
            b: "hehe".to_string(),
        },
        EmptyMutation,
        EmptySubscription,
    );
    assert_eq!(
        schema
            .execute("{ obj { fieldA fieldB } }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "obj": {
                "fieldA": "haha",
                "fieldB": "hehe",
            }
        })
    )
}

#[tokio::test]
pub async fn test_issue_539() {
    // https://github.com/async-graphql/async-graphql/issues/539#issuecomment-862209442

    struct Query;

    #[Object]
    impl Query {
        async fn value(&self) -> i32 {
            10
        }
    }

    #[derive(SimpleObject)]
    struct A {
        a: Option<Box<A>>,
    }

    #[derive(SimpleObject)]
    struct B {
        b: Option<Box<B>>,
    }

    #[derive(MergedObject)]
    pub struct Mutation(A, B);

    let schema = Schema::new(
        Query,
        Mutation(A { a: None }, B { b: None }),
        EmptySubscription,
    );
    assert_eq!(
        schema
            .execute("{ __type(name: \"Mutation\") { fields { name type { name } } } }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "__type": {
                "fields": [
                    {
                        "name": "a",
                        "type": { "name": "A" },
                    },
                    {
                        "name": "b",
                        "type": { "name": "B" },
                    }
                ]
            }
        })
    )
}

#[tokio::test]
pub async fn test_issue_694() {
    struct Query;

    #[Object]
    impl Query {
        async fn test(&self) -> bool {
            true
        }
    }

    #[derive(MergedObject)]
    pub struct QueryRoot(Query, EmptyMutation);

    let schema = Schema::new(
        QueryRoot(Query, EmptyMutation),
        EmptyMutation,
        EmptySubscription,
    );
    assert_eq!(
        schema.execute("{ test }").await.into_result().unwrap().data,
        value!({
            "test": true,
        })
    );
}
