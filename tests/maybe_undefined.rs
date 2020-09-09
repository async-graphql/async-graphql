use async_graphql::*;

#[async_std::test]
pub async fn test_maybe_undefined_type() {
    #[InputObject]
    struct MyInput {
        value: MaybeUndefined<i32>,
    }

    struct Query;

    #[Object]
    impl Query {
        async fn value1(&self, input: MaybeUndefined<i32>) -> i32 {
            if input.is_null() {
                1
            } else if input.is_undefined() {
                2
            } else {
                input.take().unwrap()
            }
        }

        async fn value2(&self, input: MyInput) -> i32 {
            if input.value.is_null() {
                1
            } else if input.value.is_undefined() {
                2
            } else {
                input.value.take().unwrap()
            }
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = r#"
        {
            v1:value1(input: 99)
            v2:value1(input: null)
            v3:value1
            v4:value2(input: { value: 99} )
            v5:value2(input: { value: null} )
            v6:value2(input: {} )
        }
    "#;
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "v1": 99,
            "v2": 1,
            "v3": 2,
            "v4": 99,
            "v5": 1,
            "v6": 2,
        })
    );
}
