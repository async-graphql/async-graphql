use async_graphql::*;

#[tokio::test]
pub async fn test_default_value_arg() {
    struct Query;

    #[Object]
    impl Query {
        async fn value1(&self, #[graphql(default = 100)] input: i32) -> i32 {
            input
        }

        async fn value2(&self, #[graphql(default)] input: i32) -> i32 {
            input
        }

        async fn value3(&self, #[graphql(default_with = "1 + 2 + 3")] input: i32) -> i32 {
            input
        }

        async fn value4(&self, #[graphql(default = 100)] input: u32) -> u32 {
            input
        }
    }

    let query = "{ value1 value2 value3 value4 }";
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "value1": 100,
            "value2": 0,
            "value3": 6,
            "value4": 100,
        })
    );

    let query = "{ value1(input: 1) value2(input: 2) value3(input: 3) }";
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "value1": 1,
            "value2": 2,
            "value3": 3,
        })
    );
}

#[tokio::test]
pub async fn test_default_value_inputobject() {
    #[derive(InputObject)]
    struct MyInput {
        #[graphql(default = 100)]
        value1: i32,

        #[graphql(default)]
        value2: i32,

        #[graphql(default_with = "1 + 2 + 3")]
        value3: i32,

        #[graphql(default = 80.0)]
        value4: f64,
    }

    #[derive(SimpleObject)]
    struct MyOutput {
        value1: i32,
        value2: i32,
        value3: i32,
        value4: f64,
    }

    struct Query;

    #[Object]
    impl Query {
        async fn value(&self, input: MyInput) -> MyOutput {
            MyOutput {
                value1: input.value1,
                value2: input.value2,
                value3: input.value3,
                value4: input.value4,
            }
        }
    }

    let query = "{ value(input: {}) { value1 value2 value3 value4 } }";
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "value": {
                "value1": 100,
                "value2": 0,
                "value3": 6,
                "value4": 80.0,
            }
        })
    );

    let query = "{ value(input: { value1: 1, value2: 2, value3: 3, value4: 88.0 }) { value1 value2 value3 value4 } }";
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "value": {
                "value1": 1,
                "value2": 2,
                "value3": 3,
                "value4": 88.0,
            }
        })
    );
}
