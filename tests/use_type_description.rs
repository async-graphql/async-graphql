use async_graphql::*;
use chrono::{DateTime, Utc};
use futures_util::stream::Stream;

#[tokio::test]
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

#[tokio::test]
pub async fn test_object_with_lifetime() {
    /// Haha
    #[derive(Description, Default)]
    struct MyObj<'a>(&'a str);

    #[Object(use_type_description)]
    impl<'a> MyObj<'a> {
        async fn value(&self) -> &str {
            self.0
        }
    }

    struct Query;

    #[Object]
    #[allow(unreachable_code)]
    impl Query {
        async fn obj(&self) -> MyObj<'_> {
            todo!()
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
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

#[tokio::test]
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

#[tokio::test]
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

#[tokio::test]
pub async fn test_override_description() {
    /// Haha
    #[derive(SimpleObject)]
    struct Query {
        value: i32,
        value2: DateTime<Utc>,
    }

    let schema = Schema::build(
        Query {
            value: 100,
            value2: Utc::now(),
        },
        EmptyMutation,
        EmptySubscription,
    )
    .override_output_type_description::<Query>("Hehe")
    .override_output_type_description::<DateTime<Utc>>("DT")
    .finish();

    assert_eq!(
        schema
            .execute(r#"{ __type(name: "Query") { description } }"#)
            .await
            .data,
        value!({
            "__type": { "description": "Hehe" }
        })
    );

    assert_eq!(
        schema
            .execute(r#"{ __type(name: "DateTime") { description } }"#)
            .await
            .data,
        value!({
            "__type": { "description": "DT" }
        })
    );
}
