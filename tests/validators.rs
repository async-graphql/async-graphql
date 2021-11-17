use async_graphql::*;
use futures_util::{Stream, StreamExt};

#[tokio::test]
pub async fn test_all_validator() {
    struct Query;

    #[Object]
    #[allow(unreachable_code, unused_variables)]
    impl Query {
        async fn multiple_of(&self, #[graphql(validator(multiple_of = 10))] n: i32) -> i32 {
            todo!()
        }

        async fn maximum(&self, #[graphql(validator(maximum = 10))] n: i32) -> i32 {
            todo!()
        }

        async fn minimum(&self, #[graphql(validator(minimum = 10))] n: i32) -> i32 {
            todo!()
        }

        async fn max_length(&self, #[graphql(validator(max_length = 10))] n: String) -> i32 {
            todo!()
        }

        async fn min_length(&self, #[graphql(validator(min_length = 10))] n: String) -> i32 {
            todo!()
        }

        async fn max_items(&self, #[graphql(validator(max_items = 10))] n: Vec<String>) -> i32 {
            todo!()
        }

        async fn min_items(&self, #[graphql(validator(min_items = 10))] n: Vec<String>) -> i32 {
            todo!()
        }

        async fn chars_max_length(
            &self,
            #[graphql(validator(chars_max_length = 10))] n: String,
        ) -> i32 {
            todo!()
        }

        async fn chars_length(
            &self,
            #[graphql(validator(chars_min_length = 10))] n: String,
        ) -> i32 {
            todo!()
        }

        async fn email(&self, #[graphql(validator(email))] n: String) -> i32 {
            todo!()
        }

        async fn list_email(&self, #[graphql(validator(list, email))] n: Vec<String>) -> i32 {
            todo!()
        }
    }
}

#[tokio::test]
pub async fn test_validator_on_object_field_args() {
    struct Query;

    #[Object]
    impl Query {
        async fn value(&self, #[graphql(validator(maximum = 10))] n: i32) -> i32 {
            n
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema
            .execute("{ value(n: 5) }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "value": 5 })
    );

    assert_eq!(
        schema
            .execute("{ value(n: 11) }")
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: r#"Failed to parse "Int": the value is 11, must be less than or equal to 10"#
                .to_string(),
            source: None,
            locations: vec![Pos {
                line: 1,
                column: 12
            }],
            path: vec![PathSegment::Field("value".to_string())],
            extensions: None
        }]
    );
}

#[tokio::test]
pub async fn test_validator_on_input_object_field() {
    #[derive(InputObject)]
    struct MyInput {
        #[graphql(validator(maximum = 10))]
        a: i32,
    }

    struct Query;

    #[Object]
    impl Query {
        async fn value(&self, input: MyInput) -> i32 {
            input.a
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema
            .execute("{ value(input: {a: 5}) }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "value": 5 })
    );

    assert_eq!(
        schema
            .execute("{ value(input: {a: 11}) }")
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: r#"Failed to parse "Int": the value is 11, must be less than or equal to 10 (occurred while parsing "MyInput")"#
                .to_string(),
            source: None,
            locations: vec![Pos {
                line: 1,
                column: 16
            }],
            path: vec![PathSegment::Field("value".to_string())],
            extensions: None
        }]
    );
}

#[tokio::test]
pub async fn test_validator_on_complex_object_field_args() {
    #[derive(SimpleObject)]
    #[graphql(complex)]
    struct Query {
        a: i32,
    }

    #[ComplexObject]
    impl Query {
        async fn value(&self, #[graphql(validator(maximum = 10))] n: i32) -> i32 {
            n
        }
    }

    let schema = Schema::new(Query { a: 10 }, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema
            .execute("{ value(n: 5) }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "value": 5 })
    );

    assert_eq!(
        schema
            .execute("{ value(n: 11) }")
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: r#"Failed to parse "Int": the value is 11, must be less than or equal to 10"#
                .to_string(),
            source: None,
            locations: vec![Pos {
                line: 1,
                column: 12
            }],
            path: vec![PathSegment::Field("value".to_string())],
            extensions: None
        }]
    );
}

#[tokio::test]
pub async fn test_validator_on_subscription_field_args() {
    struct Query;

    #[Object]
    impl Query {
        async fn value(&self) -> i32 {
            1
        }
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        async fn value(
            &self,
            #[graphql(validator(maximum = 10))] n: i32,
        ) -> impl Stream<Item = i32> {
            futures_util::stream::iter(vec![n])
        }
    }

    let schema = Schema::new(Query, EmptyMutation, Subscription);
    assert_eq!(
        schema
            .execute_stream("subscription { value(n: 5) }")
            .collect::<Vec<_>>()
            .await
            .remove(0)
            .into_result()
            .unwrap()
            .data,
        value!({ "value": 5 })
    );

    assert_eq!(
        schema
            .execute_stream("subscription { value(n: 11) }")
            .collect::<Vec<_>>()
            .await
            .remove(0)
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: r#"Failed to parse "Int": the value is 11, must be less than or equal to 10"#
                .to_string(),
            source: None,
            locations: vec![Pos {
                line: 1,
                column: 25
            }],
            path: vec![PathSegment::Field("value".to_string())],
            extensions: None
        }]
    );
}

#[tokio::test]
pub async fn test_custom_validator() {
    struct MyValidator {
        expect: i32,
    }

    impl MyValidator {
        pub fn new(n: i32) -> Self {
            MyValidator { expect: n }
        }
    }

    impl CustomValidator<i32> for MyValidator {
        fn check(&self, value: &i32) -> Result<(), String> {
            if *value == self.expect {
                Ok(())
            } else {
                Err(format!("expect 100, actual {}", value))
            }
        }
    }

    #[derive(InputObject)]
    struct MyInput {
        #[graphql(validator(custom = "MyValidator::new(100)"))]
        n: i32,
    }

    struct Query;

    #[Object]
    impl Query {
        async fn value(
            &self,
            #[graphql(validator(custom = "MyValidator::new(100)"))] n: i32,
        ) -> i32 {
            n
        }

        async fn input(&self, input: MyInput) -> i32 {
            input.n
        }
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        async fn value(
            &self,
            #[graphql(validator(custom = "MyValidator::new(100)"))] n: i32,
        ) -> impl Stream<Item = i32> {
            futures_util::stream::iter(vec![n])
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema
            .execute("{ value(n: 100) }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "value": 100 })
    );

    assert_eq!(
        schema
            .execute("{ value(n: 11) }")
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: r#"Failed to parse "Int": expect 100, actual 11"#.to_string(),
            source: None,
            locations: vec![Pos {
                line: 1,
                column: 12
            }],
            path: vec![PathSegment::Field("value".to_string())],
            extensions: None
        }]
    );

    assert_eq!(
        schema
            .execute("{ input(input: {n: 100} ) }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "input": 100 })
    );
    assert_eq!(
        schema
            .execute("{ input(input: {n: 11} ) }")
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message:
                r#"Failed to parse "Int": expect 100, actual 11 (occurred while parsing "MyInput")"#
                    .to_string(),
            source: None,
            locations: vec![Pos {
                line: 1,
                column: 16
            }],
            path: vec![PathSegment::Field("input".to_string())],
            extensions: None
        }]
    );
}

#[tokio::test]
pub async fn test_list_validator() {
    struct Query;

    #[Object]
    impl Query {
        async fn value(&self, #[graphql(validator(maximum = 3, list))] n: Vec<i32>) -> i32 {
            n.into_iter().sum()
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema
            .execute("{ value(n: [1, 2, 3]) }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "value": 6 })
    );

    assert_eq!(
        schema
            .execute("{ value(n: [1, 2, 3, 4]) }")
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: r#"Failed to parse "Int": the value is 4, must be less than or equal to 3"#
                .to_string(),
            source: None,
            locations: vec![Pos {
                line: 1,
                column: 12
            }],
            path: vec![PathSegment::Field("value".to_string())],
            extensions: None
        }]
    );
}
