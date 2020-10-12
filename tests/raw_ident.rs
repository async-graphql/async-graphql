use async_graphql::*;
use futures::{Stream, StreamExt, TryStreamExt};

#[async_std::test]
pub async fn test_input_value_custom_error() {
    #[derive(Enum, Copy, Clone, Eq, PartialEq)]
    #[allow(non_camel_case_types)]
    enum MyEnum {
        r#type,
    }

    #[derive(SimpleObject)]
    struct MyObject {
        r#i32: i32,
    }

    #[derive(InputObject)]
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
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "type": 99,
            "obj": { "i32": 88 },
            "enumValue": "TYPE",
        })
    );

    let mut stream = schema
        .execute_stream("subscription { type }")
        .map(|resp| resp.into_result())
        .map_ok(|resp| resp.data)
        .boxed();
    for i in 0..10 {
        assert_eq!(value!({ "type": i }), stream.next().await.unwrap().unwrap());
    }
    assert!(stream.next().await.is_none());
}
