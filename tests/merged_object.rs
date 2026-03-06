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
        T: Clone + OutputType,
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
        T: Clone + OutputType,
    {
        async fn events2(&self) -> impl Stream<Item = T> {
            futures_util::stream::iter(self.values.clone())
        }
    }

    #[derive(MergedSubscription)]
    struct Subscription<T1, T2>(Subscription1<T1>, Subscription2<T2>)
    where
        T1: Clone + OutputType,
        T2: Clone + OutputType;

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

/// Regression test for https://github.com/async-graphql/async-graphql/issues/1647
/// Merging 130+ objects with the old linear-chain macro would exceed the
/// compiler's default recursion limit of 128. The balanced binary tree approach
/// reduces depth from O(N) to O(log N), making this compile without increasing
/// the recursion limit.
mod issue_1647 {
    use super::*;

    macro_rules! make_simple_object {
        ($name:ident, $field:ident) => {
            #[derive(SimpleObject, Default)]
            struct $name {
                $field: i32,
            }
        };
    }

    make_simple_object!(Obj001, f001);
    make_simple_object!(Obj002, f002);
    make_simple_object!(Obj003, f003);
    make_simple_object!(Obj004, f004);
    make_simple_object!(Obj005, f005);
    make_simple_object!(Obj006, f006);
    make_simple_object!(Obj007, f007);
    make_simple_object!(Obj008, f008);
    make_simple_object!(Obj009, f009);
    make_simple_object!(Obj010, f010);
    make_simple_object!(Obj011, f011);
    make_simple_object!(Obj012, f012);
    make_simple_object!(Obj013, f013);
    make_simple_object!(Obj014, f014);
    make_simple_object!(Obj015, f015);
    make_simple_object!(Obj016, f016);
    make_simple_object!(Obj017, f017);
    make_simple_object!(Obj018, f018);
    make_simple_object!(Obj019, f019);
    make_simple_object!(Obj020, f020);
    make_simple_object!(Obj021, f021);
    make_simple_object!(Obj022, f022);
    make_simple_object!(Obj023, f023);
    make_simple_object!(Obj024, f024);
    make_simple_object!(Obj025, f025);
    make_simple_object!(Obj026, f026);
    make_simple_object!(Obj027, f027);
    make_simple_object!(Obj028, f028);
    make_simple_object!(Obj029, f029);
    make_simple_object!(Obj030, f030);
    make_simple_object!(Obj031, f031);
    make_simple_object!(Obj032, f032);
    make_simple_object!(Obj033, f033);
    make_simple_object!(Obj034, f034);
    make_simple_object!(Obj035, f035);
    make_simple_object!(Obj036, f036);
    make_simple_object!(Obj037, f037);
    make_simple_object!(Obj038, f038);
    make_simple_object!(Obj039, f039);
    make_simple_object!(Obj040, f040);
    make_simple_object!(Obj041, f041);
    make_simple_object!(Obj042, f042);
    make_simple_object!(Obj043, f043);
    make_simple_object!(Obj044, f044);
    make_simple_object!(Obj045, f045);
    make_simple_object!(Obj046, f046);
    make_simple_object!(Obj047, f047);
    make_simple_object!(Obj048, f048);
    make_simple_object!(Obj049, f049);
    make_simple_object!(Obj050, f050);
    make_simple_object!(Obj051, f051);
    make_simple_object!(Obj052, f052);
    make_simple_object!(Obj053, f053);
    make_simple_object!(Obj054, f054);
    make_simple_object!(Obj055, f055);
    make_simple_object!(Obj056, f056);
    make_simple_object!(Obj057, f057);
    make_simple_object!(Obj058, f058);
    make_simple_object!(Obj059, f059);
    make_simple_object!(Obj060, f060);
    make_simple_object!(Obj061, f061);
    make_simple_object!(Obj062, f062);
    make_simple_object!(Obj063, f063);
    make_simple_object!(Obj064, f064);
    make_simple_object!(Obj065, f065);
    make_simple_object!(Obj066, f066);
    make_simple_object!(Obj067, f067);
    make_simple_object!(Obj068, f068);
    make_simple_object!(Obj069, f069);
    make_simple_object!(Obj070, f070);
    make_simple_object!(Obj071, f071);
    make_simple_object!(Obj072, f072);
    make_simple_object!(Obj073, f073);
    make_simple_object!(Obj074, f074);
    make_simple_object!(Obj075, f075);
    make_simple_object!(Obj076, f076);
    make_simple_object!(Obj077, f077);
    make_simple_object!(Obj078, f078);
    make_simple_object!(Obj079, f079);
    make_simple_object!(Obj080, f080);
    make_simple_object!(Obj081, f081);
    make_simple_object!(Obj082, f082);
    make_simple_object!(Obj083, f083);
    make_simple_object!(Obj084, f084);
    make_simple_object!(Obj085, f085);
    make_simple_object!(Obj086, f086);
    make_simple_object!(Obj087, f087);
    make_simple_object!(Obj088, f088);
    make_simple_object!(Obj089, f089);
    make_simple_object!(Obj090, f090);
    make_simple_object!(Obj091, f091);
    make_simple_object!(Obj092, f092);
    make_simple_object!(Obj093, f093);
    make_simple_object!(Obj094, f094);
    make_simple_object!(Obj095, f095);
    make_simple_object!(Obj096, f096);
    make_simple_object!(Obj097, f097);
    make_simple_object!(Obj098, f098);
    make_simple_object!(Obj099, f099);
    make_simple_object!(Obj100, f100);
    make_simple_object!(Obj101, f101);
    make_simple_object!(Obj102, f102);
    make_simple_object!(Obj103, f103);
    make_simple_object!(Obj104, f104);
    make_simple_object!(Obj105, f105);
    make_simple_object!(Obj106, f106);
    make_simple_object!(Obj107, f107);
    make_simple_object!(Obj108, f108);
    make_simple_object!(Obj109, f109);
    make_simple_object!(Obj110, f110);
    make_simple_object!(Obj111, f111);
    make_simple_object!(Obj112, f112);
    make_simple_object!(Obj113, f113);
    make_simple_object!(Obj114, f114);
    make_simple_object!(Obj115, f115);
    make_simple_object!(Obj116, f116);
    make_simple_object!(Obj117, f117);
    make_simple_object!(Obj118, f118);
    make_simple_object!(Obj119, f119);
    make_simple_object!(Obj120, f120);
    make_simple_object!(Obj121, f121);
    make_simple_object!(Obj122, f122);
    make_simple_object!(Obj123, f123);
    make_simple_object!(Obj124, f124);
    make_simple_object!(Obj125, f125);
    make_simple_object!(Obj126, f126);
    make_simple_object!(Obj127, f127);
    make_simple_object!(Obj128, f128);
    make_simple_object!(Obj129, f129);
    make_simple_object!(Obj130, f130);

    #[rustfmt::skip] // otherwise many lines
    #[derive(MergedObject, Default)]
    struct ManyMerged(
        Obj001, Obj002, Obj003, Obj004, Obj005, Obj006, Obj007, Obj008, Obj009, Obj010,
        Obj011, Obj012, Obj013, Obj014, Obj015, Obj016, Obj017, Obj018, Obj019, Obj020,
        Obj021, Obj022, Obj023, Obj024, Obj025, Obj026, Obj027, Obj028, Obj029, Obj030,
        Obj031, Obj032, Obj033, Obj034, Obj035, Obj036, Obj037, Obj038, Obj039, Obj040,
        Obj041, Obj042, Obj043, Obj044, Obj045, Obj046, Obj047, Obj048, Obj049, Obj050,
        Obj051, Obj052, Obj053, Obj054, Obj055, Obj056, Obj057, Obj058, Obj059, Obj060,
        Obj061, Obj062, Obj063, Obj064, Obj065, Obj066, Obj067, Obj068, Obj069, Obj070,
        Obj071, Obj072, Obj073, Obj074, Obj075, Obj076, Obj077, Obj078, Obj079, Obj080,
        Obj081, Obj082, Obj083, Obj084, Obj085, Obj086, Obj087, Obj088, Obj089, Obj090,
        Obj091, Obj092, Obj093, Obj094, Obj095, Obj096, Obj097, Obj098, Obj099, Obj100,
        Obj101, Obj102, Obj103, Obj104, Obj105, Obj106, Obj107, Obj108, Obj109, Obj110,
        Obj111, Obj112, Obj113, Obj114, Obj115, Obj116, Obj117, Obj118, Obj119, Obj120,
        Obj121, Obj122, Obj123, Obj124, Obj125, Obj126, Obj127, Obj128, Obj129, Obj130,
    );

    #[tokio::test]
    pub async fn test_merged_object_many_fields() {
        struct Query;

        #[Object]
        impl Query {
            async fn obj(&self) -> ManyMerged {
                let mut m = ManyMerged::default();
                m.0 = Obj001 { f001: 1 };
                m.64 = Obj065 { f065: 65 };
                m.129 = Obj130 { f130: 130 };
                m
            }
        }

        let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
        let query = "{ obj { f001 f065 f130 } }";
        assert_eq!(
            schema.execute(query).await.into_result().unwrap().data,
            value!({
                "obj": {
                    "f001": 1,
                    "f065": 65,
                    "f130": 130,
                }
            })
        );
    }
}
