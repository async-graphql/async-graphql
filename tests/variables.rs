use async_graphql::*;
use std::collections::HashMap;

#[tokio::test]
pub async fn test_variables() {
    struct Query;

    #[Object]
    impl Query {
        pub async fn int_val(&self, value: i32) -> i32 {
            value
        }

        pub async fn int_list_val(&self, value: Vec<i32>) -> Vec<i32> {
            value
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = Request::new(
        r#"
            query QueryWithVariables($intVal: Int!, $intListVal: [Int!]!) {
                intVal(value: $intVal)
                intListVal(value: $intListVal)
            }
        "#,
    )
    .variables(Variables::from_value(value!({
        "intVal": 10,
         "intListVal": [1, 2, 3, 4, 5],
    })));

    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "intVal": 10,
            "intListVal": [1, 2, 3, 4, 5],
        })
    );
}

#[tokio::test]
pub async fn test_variable_default_value() {
    struct Query;

    #[Object]
    impl Query {
        pub async fn int_val(&self, value: i32) -> i32 {
            value
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema
            .execute(
                r#"
            query QueryWithVariables($intVal: Int = 10) {
                intVal(value: $intVal)
            }
        "#
            )
            .await
            .data,
        value!({
            "intVal": 10,
        })
    );
}

#[tokio::test]
pub async fn test_variable_no_value() {
    struct Query;

    #[Object]
    impl Query {
        pub async fn int_val(&self, value: Option<i32>) -> i32 {
            value.unwrap_or(10)
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let resp = schema
        .execute(Request::new(
            r#"
            query QueryWithVariables($intVal: Int) {
                intVal(value: $intVal)
            }
        "#,
        ))
        .await
        .into_result()
        .unwrap();
    assert_eq!(
        resp.data,
        value!({
            "intVal": 10,
        })
    );
}

#[tokio::test]
pub async fn test_variable_null() {
    struct Query;

    #[Object]
    impl Query {
        pub async fn int_val(&self, value: Option<i32>) -> i32 {
            value.unwrap_or(10)
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = Request::new(
        r#"
            query QueryWithVariables($intVal: Int) {
                intVal(value: $intVal)
            }
        "#,
    )
    .variables(Variables::from_value(value!({
        "intVal": null,
    })));
    let resp = schema.execute(query).await;
    assert_eq!(
        resp.data,
        value!({
            "intVal": 10,
        })
    );
}

#[tokio::test]
pub async fn test_variable_in_input_object() {
    #[derive(InputObject)]
    struct MyInput {
        value: i32,
    }

    struct Query;

    #[Object]
    impl Query {
        async fn test(&self, input: MyInput) -> i32 {
            input.value
        }

        async fn test2(&self, input: Vec<MyInput>) -> i32 {
            input.iter().map(|item| item.value).sum()
        }
    }

    struct Mutation;

    #[Object]
    impl Mutation {
        async fn test(&self, input: MyInput) -> i32 {
            input.value
        }
    }

    let schema = Schema::new(Query, Mutation, EmptySubscription);

    // test query
    {
        let query = r#"
        query TestQuery($value: Int!) {
            test(input: {value: $value })
        }"#;
        let resp = schema
            .execute(Request::new(query).variables(Variables::from_value(value!({
                "value": 10,
            }))))
            .await;
        assert_eq!(
            resp.data,
            value!({
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
        let resp = schema
            .execute(Request::new(query).variables(Variables::from_value(value!({
                "value": 3,
            }))))
            .await;
        assert_eq!(
            resp.data,
            value!({
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
        let resp = schema
            .execute(Request::new(query).variables(Variables::from_value(value!({
                "value": 10,
            }))))
            .await;
        assert_eq!(
            resp.data,
            value!({
                "test": 10,
            })
        );
    }
}

#[tokio::test]
pub async fn test_variables_enum() {
    #[derive(Enum, Eq, PartialEq, Copy, Clone)]
    enum MyEnum {
        A,
        B,
        C,
    }

    struct Query;

    #[Object]
    impl Query {
        pub async fn value(&self, value: MyEnum) -> i32 {
            match value {
                MyEnum::A => 1,
                MyEnum::B => 2,
                MyEnum::C => 3,
            }
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = Request::new(
        r#"
            query QueryWithVariables($value1: MyEnum, $value2: MyEnum, $value3: MyEnum) {
                a: value(value: $value1)
                b: value(value: $value2)
                c: value(value: $value3)
            }
        "#,
    )
    .variables(Variables::from_value(value!({
        "value1": "A",
        "value2": "B",
        "value3": "C",
    })));

    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "a": 1,
            "b": 2,
            "c": 3,
        })
    );
}

#[tokio::test]
pub async fn test_variables_json() {
    struct Query;

    #[Object]
    impl Query {
        pub async fn value(&self, value: Json<HashMap<String, i32>>) -> i32 {
            *value.get("a-b").unwrap()
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = Request::new(
        r#"
            query QueryWithVariables($value: JSON) {
                value(value: $value)
            }
        "#,
    )
    .variables(Variables::from_value(value!({
        "value": { "a-b": 123 },
    })));

    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "value": 123,
        })
    );
}

#[tokio::test]
pub async fn test_variables_invalid_type() {
    struct Query;

    #[Object]
    impl Query {
        pub async fn int_val(&self, value: Option<i32>) -> i32 {
            value.unwrap_or(10)
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = Request::new(
        r#"
            query QueryWithVariables($intVal: invalid) {
                intVal(value: $intVal)
            }
        "#,
    )
    .variables(Variables::from_value(value!({
        "intVal": null,
    })));
    let resp = schema.execute(query).await;
    assert_eq!(
        resp.errors.first().map(|v| v.message.as_str()),
        Some("Unknown type \"invalid\"")
    );
}

#[tokio::test]
pub async fn test_variables_invalid_type_with_value() {
    struct Query;

    #[Object]
    impl Query {
        pub async fn int_val(&self, value: Option<i32>) -> i32 {
            value.unwrap_or(10)
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = Request::new(
        r#"
            query QueryWithVariables($intVal: invalid = 2) {
                intVal(value: $intVal)
            }
        "#,
    )
    .variables(Variables::from_value(value!({
        "intVal": null,
    })));
    let resp = schema.execute(query).await;
    assert_eq!(
        resp.errors.first().map(|v| v.message.as_str()),
        Some("Unknown type \"invalid\"")
    );
}
