use async_graphql::*;
use futures::{Stream, StreamExt};

#[async_std::test]
pub async fn test_input_value_custom_error() {
    #[Enum]
    #[allow(non_camel_case_types)]
    enum MyEnum {
        r#type,
    }

    #[SimpleObject]
    struct MyObject {
        r#i32: i32,
    }

    #[InputObject]
    struct MyInputObject {
        r#i32: i32,
    }

    struct Query;

    #[Object]
    impl Query {
        async fn r#type(&self, r#i32: i32) -> i32 {
            r#i32
        }

        async fn obj(&self, obj: MyInputObject) -> MyObject {
            MyObject { r#i32: obj.r#i32 }
        }

        async fn enum_value(&self, value: MyEnum) -> MyEnum {
            value
        }
    }

    struct SubscriptionRoot;

    #[Subscription]
    impl SubscriptionRoot {
        async fn r#type(&self) -> impl Stream<Item = i32> {
            futures::stream::iter(0..10)
        }
    }

    let schema = Schema::new(Query, EmptyMutation, SubscriptionRoot);
    let query = r#"
        {
            type(i32: 99)
            obj(obj: { i32: 88} ) { i32 }
            enumValue(value: TYPE)
        }"#;
    assert_eq!(
        QueryBuilder::new(query)
            .execute(&schema)
            .await
            .unwrap()
            .data,
        serde_json::json!({
            "type": 99,
            "obj": { "i32": 88 },
            "enumValue": "TYPE",
        })
    );

    let mut stream = schema
        .create_subscription_stream("subscription { type }", None, Default::default(), None)
        .await
        .unwrap();
    for i in 0..10 {
        assert_eq!(
            Some(Ok(serde_json::json!({ "type": i }))),
            stream.next().await
        );
    }
    assert!(stream.next().await.is_none());
}
