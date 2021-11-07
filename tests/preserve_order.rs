use async_graphql::*;

#[tokio::test]
pub async fn test_preserve_order() {
    #[derive(SimpleObject)]
    struct Root {
        a: i32,
        b: i32,
        c: i32,
    }

    let schema = Schema::new(Root { a: 1, b: 2, c: 3 }, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema
            .execute("{ a c b }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "a": 1, "c": 3, "b": 2
        })
    );
    assert_eq!(
        serde_json::to_string(
            &schema
                .execute("{ a c b }")
                .await
                .into_result()
                .unwrap()
                .data
        )
        .unwrap(),
        r#"{"a":1,"c":3,"b":2}"#
    );

    assert_eq!(
        schema
            .execute("{ c b a }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "c": 3, "b": 2, "a": 1
        })
    );
    assert_eq!(
        serde_json::to_string(
            &schema
                .execute("{ c b a }")
                .await
                .into_result()
                .unwrap()
                .data
        )
        .unwrap(),
        r#"{"c":3,"b":2,"a":1}"#
    );
}
