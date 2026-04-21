#[cfg(test)]
mod tests {
    use async_graphql::*;

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

    #[tokio::test]
    pub async fn test_maybe_undefined_type() {
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
    }

    #[tokio::test]
    pub async fn test_maybe_undefined_in_operation() {
        let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
        let query_operation = r#"
            query TestQuery($input: Int) {
                v1:value1(input: null)
                v2:value1(input: 99)
                v3:value1(input: $input)
            }
        "#;
        assert_eq!(
            schema.execute(query_operation).await.data,
            value!({
                "v1": 1,
                "v2": 99,
                "v3": 2,
            })
        );
    }

    #[tokio::test]
    pub async fn test_maybe_undefined_with_variables() {
        // An omitted variable should stay undefined instead of being coerced to `null`.
        let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
        let query = Request::new(
            r#"
                query TestQuery($input: Int, $input2: Int, $input3: Int) {
                    v1:value1(input: $input)
                    v2:value1(input: $input2)
                    v3:value1(input: $input3)
                }
            "#,
        )
        .variables(Variables::from_value(value!({
            "input": 10,
            "input2": null,
        })));
        assert_eq!(
            schema.execute(query).await.data,
            value!({
                "v1": 10,
                "v2": 1,
                "v3": 2,
            })
        );
    }

    #[tokio::test]
    pub async fn test_maybe_undefined_input_object_with_variables() {
        // Omitted variables inside input objects should leave the field undefined.
        let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
        let query = Request::new(
            r#"
                query TestQuery($input: Int, $input2: Int, $input3: Int) {
                    v1:value2(input: { value: $input })
                    v2:value2(input: { value: $input2 })
                    v3:value2(input: { value: $input3 })
                }
            "#,
        )
        .variables(Variables::from_value(value!({
            "input": 10,
            "input2": null,
        })));
        assert_eq!(
            schema.execute(query).await.data,
            value!({
                "v1": 10,
                "v2": 1,
                "v3": 2,
            })
        );
    }
}
