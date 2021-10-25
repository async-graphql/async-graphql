use async_graphql::*;

#[tokio::test]
pub async fn test_derived_field_object() {
    use serde::{Deserialize, Serialize};

    struct Query;

    #[derive(Serialize, Deserialize)]
    struct ValueDerived(String);

    scalar!(ValueDerived);

    impl From<i32> for ValueDerived {
        fn from(value: i32) -> Self {
            ValueDerived(format!("{}", value))
        }
    }

    #[Object]
    impl Query {
        #[graphql(derived(name = "value2", into = "ValueDerived"))]
        async fn value1(&self, #[graphql(default = 100)] input: i32) -> i32 {
            input
        }
    }

    let query = "{ value1 value2 }";
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "value1": 100,
            "value2": "100",
        })
    );

    let query = "{ value1(input: 1) value2(input: 2) }";
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "value1": 1,
            "value2": "2",
        })
    );
}

#[tokio::test]
pub async fn test_derived_field_simple_object() {
    use serde::{Deserialize, Serialize};

    struct Query;

    #[derive(Serialize, Deserialize)]
    struct ValueDerived(String);

    scalar!(ValueDerived);

    impl From<i32> for ValueDerived {
        fn from(value: i32) -> Self {
            ValueDerived(format!("{}", value))
        }
    }

    #[derive(SimpleObject)]
    struct TestObj {
        #[graphql(owned, derived(name = "value2", into = "ValueDerived"))]
        pub value1: i32,
    }

    #[Object]
    impl Query {
        async fn test(&self, #[graphql(default = 100)] input: i32) -> TestObj {
            TestObj { value1: input }
        }
    }

    let query = "{ test { value1 value2 } }";
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "test": {
                "value1": 100,
                "value2": "100",
            }
        })
    );

    let query = "{ test(input: 2) { value1 value2 }}";
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    dbg!(schema.execute(query).await);
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "test": {
                "value1": 2,
                "value2": "2",
            }
        })
    );
}
