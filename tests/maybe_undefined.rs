use async_graphql::*;

#[tokio::test]
pub async fn test_maybe_undefined_type() {
    #[derive(InputObject)]
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
        schema.execute(query).await.data,
        value!({
            "v1": 99,
            "v2": 1,
            "v3": 2,
            "v4": 99,
            "v5": 1,
            "v6": 2,
        })
    );

    let query_operation = r#"
        query TestQuery($input: Int) {
            v1:value1(input: null)
            v2:value1(input: $input)
            v3:value1(input: 99)
        }
    "#;
    assert_eq!(
        schema.execute(query_operation).await.data,
        value!({
            "v1": 1,
            "v2": 2,
            "v3": 99,
        })
    );

    let query_with_value = Request::new(
        r#"
            query TestQuery($input: Int) {
                v1:value1(input: $input)
            }
        "#,
    )
    .variables(Variables::from_value(value!({
        "input": 10,
    })));
    assert_eq!(
        schema.execute(query_with_value).await.data,
        value!({
            "v1": 10,
        })
    );

    let query_with_null = Request::new(
        r#"
            query TestQuery($input: Int) {
                v1:value1(input: $input)
            }
        "#,
    )
    .variables(Variables::from_value(value!({
        "input": null,
    })));
    assert_eq!(
        schema.execute(query_with_null).await.data,
        value!({
            "v1": 1,
        })
    );

    let query_with_undefined = Request::new(
        r#"
            query TestQuery($input: Int) {
                v1:value1(input: $input)
            }
        "#,
    )
    .variables(Variables::from_value(value!({})));
    assert_eq!(
        schema.execute(query_with_undefined).await.data,
        value!({
            "v1": 2,
        })
    );
}
