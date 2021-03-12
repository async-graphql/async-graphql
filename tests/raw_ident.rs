use async_graphql::*;
use futures_util::stream::{Stream, StreamExt, TryStreamExt};

#[tokio::test]
pub async fn test_input_value_custom_error() {
    #[derive(Enum, Copy, Clone, Eq, PartialEq)]
    #[allow(non_camel_case_types)]
    enum MyEnum {
        r#type,
    }

    #[derive(SimpleObject)]
    struct MyObject {
        r#match: i32,
    }

    #[derive(InputObject)]
    struct MyInputObject {
        r#match: i32,
    }

    struct Query;

    #[Object]
    impl Query {
        async fn r#type(&self, r#match: i32) -> i32 {
            r#match
        }

        async fn obj(&self, obj: MyInputObject) -> MyObject {
            MyObject {
                r#match: obj.r#match,
            }
        }

        async fn enum_value(&self, value: MyEnum) -> MyEnum {
            value
        }
    }

    struct SubscriptionRoot;

    #[Subscription]
    impl SubscriptionRoot {
        async fn r#type(&self) -> impl Stream<Item = i32> {
            futures_util::stream::iter(0..10)
        }
    }

    let schema = Schema::new(Query, EmptyMutation, SubscriptionRoot);
    let query = r#"
        {
            type(match: 99)
            obj(obj: { match: 88} ) { match }
            enumValue(value: TYPE)
        }"#;
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "type": 99,
            "obj": { "match": 88 },
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
