use async_graphql::*;

#[async_std::test]
pub async fn test_variables() {
    struct QueryRoot;

    #[Object]
    impl QueryRoot {
        pub async fn int_val(&self, value: i32) -> i32 {
            value
        }

        pub async fn int_list_val(&self, value: Vec<i32>) -> Vec<i32> {
            value
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
    let query = QueryBuilder::new(
        r#"
            query QueryWithVariables($intVal: Int!, $intListVal: [Int!]!) {
                intVal(value: $intVal)
                intListVal(value: $intListVal)
            }
        "#,
    )
    .variables(Variables::parse_from_json(serde_json::json!({
        "intVal": 10,
         "intListVal": [1, 2, 3, 4, 5],
    })));
    let resp = query.execute(&schema).await.unwrap();
    assert_eq!(
        resp.data,
        serde_json::json!({
            "intVal": 10,
            "intListVal": [1, 2, 3, 4, 5],
        })
    );
}

#[async_std::test]
pub async fn test_variable_default_value() {
    struct QueryRoot;

    #[Object]
    impl QueryRoot {
        pub async fn int_val(&self, value: i32) -> i32 {
            value
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
    let resp = schema
        .execute(
            r#"
            query QueryWithVariables($intVal: Int = 10) {
                intVal(value: $intVal)
            }
        "#,
        )
        .await
        .unwrap();
    assert_eq!(
        resp.data,
        serde_json::json!({
            "intVal": 10,
        })
    );
}

#[async_std::test]
pub async fn test_variable_no_value() {
    struct QueryRoot;

    #[Object]
    impl QueryRoot {
        pub async fn int_val(&self, value: Option<i32>) -> i32 {
            value.unwrap_or(10)
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
    let query = QueryBuilder::new(
        r#"
            query QueryWithVariables($intVal: Int) {
                intVal(value: $intVal)
            }
        "#,
    )
    .variables(Variables::parse_from_json(serde_json::json!({})));
    let resp = query.execute(&schema).await.unwrap();
    assert_eq!(
        resp.data,
        serde_json::json!({
            "intVal": 10,
        })
    );
}

#[async_std::test]
pub async fn test_variable_null() {
    struct QueryRoot;

    #[Object]
    impl QueryRoot {
        pub async fn int_val(&self, value: Option<i32>) -> i32 {
            value.unwrap_or(10)
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
    let query = QueryBuilder::new(
        r#"
            query QueryWithVariables($intVal: Int) {
                intVal(value: $intVal)
            }
        "#,
    )
    .variables(Variables::parse_from_json(serde_json::json!({
        "intVal": null,
    })));
    let resp = query.execute(&schema).await.unwrap();
    assert_eq!(
        resp.data,
        serde_json::json!({
            "intVal": 10,
        })
    );
}

#[async_std::test]
pub async fn test_variable_in_input_object() {
    #[InputObject]
    struct MyInput {
        value: i32,
    }

    struct QueryRoot;

    #[Object]
    impl QueryRoot {
        async fn test(&self, input: MyInput) -> i32 {
            input.value
        }

        async fn test2(&self, input: Vec<MyInput>) -> i32 {
            input.iter().map(|item| item.value).sum()
        }
    }

    struct MutationRoot;

    #[Object]
    impl MutationRoot {
        async fn test(&self, input: MyInput) -> i32 {
            input.value
        }
    }

    let schema = Schema::new(QueryRoot, MutationRoot, EmptySubscription);

    // test query
    {
        let query = r#"
        query TestQuery($value: Int!) {
            test(input: {value: $value })
        }"#;
        let resp = QueryBuilder::new(query)
            .variables(Variables::parse_from_json(serde_json::json!({
                "value": 10,
            })))
            .execute(&schema)
            .await
            .unwrap();
        assert_eq!(
            resp.data,
            serde_json::json!({
                "test": 10,
            })
        );
    }

    // test query2
    {
        let query = r#"
        query TestQuery($value: Int!) {
            test2(input: [{value: $value }, {value: $value }])
        }"#;
        let resp = QueryBuilder::new(query)
            .variables(Variables::parse_from_json(serde_json::json!({
                "value": 3,
            })))
            .execute(&schema)
            .await
            .unwrap();
        assert_eq!(
            resp.data,
            serde_json::json!({
                "test2": 6,
            })
        );
    }

    // test mutation
    {
        let query = r#"
        mutation TestMutation($value: Int!) {
            test(input: {value: $value })
        }"#;
        let resp = QueryBuilder::new(query)
            .variables(Variables::parse_from_json(serde_json::json!({
                "value": 10,
            })))
            .execute(&schema)
            .await
            .unwrap();
        assert_eq!(
            resp.data,
            serde_json::json!({
                "test": 10,
            })
        );
    }
}
