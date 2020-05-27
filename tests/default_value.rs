use async_graphql::*;

#[async_std::test]
pub async fn test_default_value_arg() {
    struct Query;

    #[Object]
    impl Query {
        async fn value1(&self, #[arg(default = 100)] input: i32) -> i32 {
            input
        }

        async fn value2(&self, #[arg(default)] input: i32) -> i32 {
            input
        }

        async fn value3(&self, #[arg(default_with = "1 + 2 + 3")] input: i32) -> i32 {
            input
        }
    }

    let query = "{ value1 value2 value3 }";
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "value1": 100,
            "value2": 0,
            "value3": 6,
        })
    );

    let query = "{ value1(input: 1) value2(input: 2) value3(input: 3) }";
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "value1": 1,
            "value2": 2,
            "value3": 3,
        })
    );
}

#[async_std::test]
pub async fn test_default_value_inputobject() {
    #[InputObject]
    struct MyInput {
        #[field(default = 100)]
        value1: i32,

        #[field(default)]
        value2: i32,

        #[field(default_with = "1 + 2 + 3")]
        value3: i32,
    }

    #[SimpleObject]
    struct MyOutput {
        value1: i32,
        value2: i32,
        value3: i32,
    }

    struct Query;

    #[Object]
    impl Query {
        async fn value(&self, input: MyInput) -> MyOutput {
            MyOutput {
                value1: input.value1,
                value2: input.value2,
                value3: input.value3,
            }
        }
    }

    let query = "{ value(input: {}) { value1 value2 value3 } }";
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "value": {
                "value1": 100,
                "value2": 0,
                "value3": 6,
            }
        })
    );

    let query = "{ value(input: { value1: 1, value2: 2, value3: 3 }) { value1 value2 value3 } }";
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "value": {
                "value1": 1,
                "value2": 2,
                "value3": 3,
            }
        })
    );
}
