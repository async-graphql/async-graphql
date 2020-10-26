use async_graphql::*;
use async_std::stream::Stream;

#[async_std::test]
pub async fn test_object() {
    /// Haha
    #[derive(Description, Default)]
    struct MyObj;

    #[Object(use_type_description)]
    impl MyObj {
        async fn value(&self) -> i32 {
            100
        }
    }

    #[derive(SimpleObject, Default)]
    struct Query {
        obj: MyObj,
    }

    let schema = Schema::new(Query::default(), EmptyMutation, EmptySubscription);
    assert_eq!(
        schema
            .execute(r#"{ __type(name: "MyObj") { description } }"#)
            .await
            .data,
        value!({
            "__type": { "description": "Haha" }
        })
    );
}

#[async_std::test]
pub async fn test_scalar() {
    /// Haha
    #[derive(Description, Default)]
    struct MyScalar(i32);

    #[Scalar(use_type_description)]
    impl ScalarType for MyScalar {
        fn parse(_value: Value) -> InputValueResult<Self> {
            Ok(MyScalar(42))
        }

        fn to_value(&self) -> Value {
            Value::Number(self.0.into())
        }
    }

    #[derive(SimpleObject, Default)]
    struct Query {
        obj: MyScalar,
    }

    let schema = Schema::new(Query::default(), EmptyMutation, EmptySubscription);
    assert_eq!(
        schema
            .execute(r#"{ __type(name: "MyScalar") { description } }"#)
            .await
            .data,
        value!({
            "__type": { "description": "Haha" }
        })
    );
}

#[async_std::test]
pub async fn test_subscription() {
    struct Query;

    #[Object]
    impl Query {
        async fn value(&self) -> i32 {
            100
        }
    }

    /// Haha
    #[derive(Description, Default)]
    struct Subscription;

    #[Subscription(use_type_description)]
    impl Subscription {
        async fn values(&self) -> impl Stream<Item = i32> {
            futures_util::stream::once(async move { 100 })
        }
    }

    let schema = Schema::new(Query, EmptyMutation, Subscription);
    assert_eq!(
        schema
            .execute(r#"{ __type(name: "Subscription") { description } }"#)
            .await
            .data,
        value!({
            "__type": { "description": "Haha" }
        })
    );
}
