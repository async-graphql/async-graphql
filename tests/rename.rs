use async_graphql::*;
use futures::{Stream, StreamExt};

#[async_std::test]
pub async fn test_enum() {
    #[derive(Enum, Eq, PartialEq, Copy, Clone)]
    #[graphql(rename_items = "camelCase")]
    #[allow(non_camel_case_types)]
    enum MyEnum {
        CREATE_OBJECT,
    }

    #[derive(SimpleObject)]
    struct Query {
        value: MyEnum,
    }

    assert_eq!(
        Schema::new(
            Query {
                value: MyEnum::CREATE_OBJECT
            },
            EmptyMutation,
            EmptySubscription
        )
        .execute("{ value }")
        .await
        .into_result()
        .unwrap()
        .data,
        value!({"value": "createObject"})
    );
}

#[async_std::test]
pub async fn test_simple_object() {
    #[derive(SimpleObject)]
    #[graphql(rename_fields = "UPPERCASE")]
    struct Query {
        a: i32,
    }

    assert_eq!(
        Schema::new(Query { a: 100 }, EmptyMutation, EmptySubscription)
            .execute("{ A }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({"A": 100})
    );
}

#[async_std::test]
pub async fn test_object() {
    struct Query;

    #[Object(rename_fields = "UPPERCASE", rename_args = "PascalCase")]
    impl Query {
        async fn a(&self, ab1_cd2: i32) -> i32 {
            100 + ab1_cd2
        }
    }

    assert_eq!(
        Schema::new(Query, EmptyMutation, EmptySubscription)
            .execute("{ A(Ab1Cd2:10) }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({"A": 110})
    );
}

#[async_std::test]
pub async fn test_input_object() {
    #[derive(InputObject)]
    #[graphql(rename_fields = "snake_case")]
    #[allow(non_snake_case)]
    struct Obj {
        a: i32,
        AbCd: i32,
    }

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self, obj: Obj) -> i32 {
            obj.a + obj.AbCd
        }
    }

    assert_eq!(
        Schema::new(Query, EmptyMutation, EmptySubscription)
            .execute("{ obj(obj: {a: 10, ab_cd: 30}) }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({"obj": 40})
    );
}

#[async_std::test]
pub async fn test_subscription() {
    #[derive(SimpleObject)]
    struct Query;

    struct Subscription;

    #[Subscription(rename_fields = "SCREAMING_SNAKE_CASE", rename_args = "lowercase")]
    #[allow(non_snake_case)]
    impl Subscription {
        async fn create_object(&self, ObjectId: i32) -> impl Stream<Item = i32> {
            futures::stream::once(async move { ObjectId })
        }
    }

    assert_eq!(
        Schema::new(Query, EmptyMutation, Subscription)
            .execute_stream("subscription { CREATE_OBJECT(objectid: 100) }")
            .boxed()
            .next()
            .await
            .unwrap()
            .into_result()
            .unwrap()
            .data,
        value!({"CREATE_OBJECT": 100})
    );
}
