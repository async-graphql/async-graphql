#![allow(clippy::uninlined_format_args)]

use std::sync::Arc;

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

        async fn url(&self, #[graphql(validator(url))] n: String) -> i32 {
            todo!()
        }

        async fn ip(&self, #[graphql(validator(ip))] n: String) -> i32 {
            todo!()
        }

        async fn regex(&self, #[graphql(validator(regex = "^[0-9]+$"))] n: String) -> i32 {
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
        #[graphql(validator(maximum = 10))]
        b: Option<i32>,
    }

    struct Query;

    #[Object]
    impl Query {
        async fn value(&self, input: MyInput) -> i32 {
            input.a + input.b.unwrap_or_default()
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

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema
            .execute("{ value(input: {a: 5, b: 7}) }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "value": 12 })
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

    assert_eq!(
        schema
            .execute("{ value(input: {a: 5, b: 20}) }")
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: r#"Failed to parse "Int": the value is 20, must be less than or equal to 10 (occurred while parsing "MyInput")"#
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
        fn check(&self, value: &i32) -> Result<(), InputValueError<i32>> {
            if *value == self.expect {
                Ok(())
            } else {
                Err(InputValueError::custom(format!(
                    "expect 100, actual {}",
                    value
                )))
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

        async fn value2(
            &self,
            #[graphql(validator(list, custom = "MyValidator::new(100)"))] values: Vec<i32>,
        ) -> i32 {
            values.into_iter().sum()
        }

        async fn value3(
            &self,
            #[graphql(validator(list, custom = "MyValidator::new(100)"))] values: Option<Vec<i32>>,
        ) -> i32 {
            values.into_iter().flatten().sum()
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

    let schema = Schema::new(Query, EmptyMutation, Subscription);
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

    assert_eq!(
        schema
            .execute_stream("subscription { value(n: 100 ) }")
            .next()
            .await
            .unwrap()
            .into_result()
            .unwrap()
            .data,
        value!({ "value": 100 })
    );

    assert_eq!(
        schema
            .execute_stream("subscription { value(n: 11 ) }")
            .next()
            .await
            .unwrap()
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: r#"Failed to parse "Int": expect 100, actual 11"#.to_string(),
            source: None,
            locations: vec![Pos {
                line: 1,
                column: 25
            }],
            path: vec![PathSegment::Field("value".to_string())],
            extensions: None
        }]
    );

    assert_eq!(
        schema
            .execute("{ value2(values: [77, 88] ) }")
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: r#"Failed to parse "Int": expect 100, actual 77"#.to_string(),
            source: None,
            locations: vec![Pos {
                line: 1,
                column: 18
            }],
            path: vec![PathSegment::Field("value2".to_string())],
            extensions: None
        }]
    );

    assert_eq!(
        schema
            .execute("{ value3(values: [77, 88] ) }")
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: r#"Failed to parse "Int": expect 100, actual 77"#.to_string(),
            source: None,
            locations: vec![Pos {
                line: 1,
                column: 18
            }],
            path: vec![PathSegment::Field("value3".to_string())],
            extensions: None
        }]
    );

    assert_eq!(
        schema
            .execute("{ value3(values: null ) }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "value3": 0
        })
    );
}

#[tokio::test]
pub async fn test_custom_validator_with_fn() {
    fn check_100(value: &i32) -> Result<(), String> {
        if *value == 100 {
            Ok(())
        } else {
            Err(format!("expect 100, actual {}", value))
        }
    }

    struct Query;

    #[Object]
    impl Query {
        async fn value(&self, #[graphql(validator(custom = "check_100"))] n: i32) -> i32 {
            n
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
}

#[tokio::test]
pub async fn test_custom_validator_with_extensions() {
    struct MyValidator {
        expect: i32,
    }

    impl MyValidator {
        pub fn new(n: i32) -> Self {
            MyValidator { expect: n }
        }
    }

    impl CustomValidator<i32> for MyValidator {
        fn check(&self, value: &i32) -> Result<(), InputValueError<i32>> {
            if *value == self.expect {
                Ok(())
            } else {
                Err(
                    InputValueError::custom(format!("expect 100, actual {}", value))
                        .with_extension("code", 99),
                )
            }
        }
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

    let mut error_extensions = ErrorExtensionValues::default();
    error_extensions.set("code", 99);
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
            extensions: Some(error_extensions)
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

#[tokio::test]
pub async fn test_validate_wrapper_types() {
    #[derive(NewType)]
    struct Size(i32);

    struct Query;

    #[Object]
    impl Query {
        async fn a(&self, #[graphql(validator(maximum = 10))] n: Option<i32>) -> i32 {
            n.unwrap_or_default()
        }

        async fn b(&self, #[graphql(validator(maximum = 10))] n: Option<Option<i32>>) -> i32 {
            n.unwrap_or_default().unwrap_or_default()
        }

        async fn c(&self, #[graphql(validator(maximum = 10))] n: MaybeUndefined<i32>) -> i32 {
            n.take().unwrap_or_default()
        }

        async fn d(&self, #[graphql(validator(maximum = 10))] n: Box<i32>) -> i32 {
            *n
        }

        async fn e(&self, #[graphql(validator(maximum = 10))] n: Arc<i32>) -> i32 {
            *n
        }

        async fn f(&self, #[graphql(validator(maximum = 10))] n: Json<i32>) -> i32 {
            n.0
        }

        async fn g(&self, #[graphql(validator(maximum = 10))] n: Option<Json<i32>>) -> i32 {
            n.map(|n| n.0).unwrap_or_default()
        }

        async fn h(&self, #[graphql(validator(maximum = 10))] n: Size) -> i32 {
            n.0
        }

        async fn i(&self, #[graphql(validator(list, maximum = 10))] n: Vec<i32>) -> i32 {
            n.into_iter().sum()
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    let successes = [
        ("{ a(n: 5) }", value!({ "a": 5 })),
        ("{ a }", value!({ "a": 0 })),
        ("{ b(n: 5) }", value!({ "b": 5 })),
        ("{ b }", value!({ "b": 0 })),
        ("{ c(n: 5) }", value!({ "c": 5 })),
        ("{ c(n: null) }", value!({ "c": 0 })),
        ("{ c }", value!({ "c": 0 })),
        ("{ d(n: 5) }", value!({ "d": 5 })),
        ("{ e(n: 5) }", value!({ "e": 5 })),
        ("{ f(n: 5) }", value!({ "f": 5 })),
        ("{ g(n: 5) }", value!({ "g": 5 })),
        ("{ g }", value!({ "g": 0 })),
        ("{ h(n: 5) }", value!({ "h": 5 })),
        ("{ i(n: [1, 2, 3]) }", value!({ "i": 6 })),
    ];

    for (query, res) in successes {
        assert_eq!(schema.execute(query).await.into_result().unwrap().data, res);
    }

    assert_eq!(
        schema
            .execute("{ a(n:20) }")
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: r#"Failed to parse "Int": the value is 20, must be less than or equal to 10"#
                .to_string(),
            source: None,
            locations: vec![Pos { line: 1, column: 7 }],
            path: vec![PathSegment::Field("a".to_string())],
            extensions: None
        }]
    );

    assert_eq!(
        schema
            .execute("{ b(n:20) }")
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: r#"Failed to parse "Int": the value is 20, must be less than or equal to 10"#
                .to_string(),
            source: None,
            locations: vec![Pos { line: 1, column: 7 }],
            path: vec![PathSegment::Field("b".to_string())],
            extensions: None
        }]
    );

    assert_eq!(
        schema
            .execute("{ f(n:20) }")
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: r#"Failed to parse "Int": the value is 20, must be less than or equal to 10"#
                .to_string(),
            source: None,
            locations: vec![Pos { line: 1, column: 7 }],
            path: vec![PathSegment::Field("f".to_string())],
            extensions: None
        }]
    );
}

#[tokio::test]
pub async fn test_list_both_max_items_and_max_length() {
    struct Query;

    #[Object]
    impl Query {
        async fn value(
            &self,
            #[graphql(validator(list, max_length = 3, max_items = 2))] values: Vec<String>,
        ) -> String {
            values.into_iter().collect()
        }

        async fn value2(
            &self,
            #[graphql(validator(list, max_length = 3, max_items = 2))] values: Option<Vec<String>>,
        ) -> String {
            values.into_iter().flatten().collect()
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema
            .execute(r#"{ value(values: ["a", "b", "cdef"])}"#)
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: r#"Failed to parse "[String!]": the value length is 3, must be less than or equal to 2"#.to_string(),
            source: None,
            locations: vec![Pos { column: 17, line: 1}],
            path: vec![PathSegment::Field("value".to_string())],
            extensions: None
        }]
    );

    assert_eq!(
        schema
            .execute(r#"{ value(values: ["a", "cdef"])}"#)
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: r#"Failed to parse "String": the string length is 4, must be less than or equal to 3"#.to_string(),
            source: None,
            locations: vec![Pos { column: 17, line: 1}],
            path: vec![PathSegment::Field("value".to_string())],
            extensions: None
        }]
    );

    assert_eq!(
        schema
            .execute(r#"{ value(values: ["a", "b"])}"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "value": "ab"
        })
    );

    assert_eq!(
        schema
            .execute(r#"{ value2(values: ["a", "b", "cdef"])}"#)
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: r#"Failed to parse "[String!]": the value length is 3, must be less than or equal to 2"#.to_string(),
            source: None,
            locations: vec![Pos { column: 18, line: 1}],
            path: vec![PathSegment::Field("value2".to_string())],
            extensions: None
        }]
    );

    assert_eq!(
        schema
            .execute(r#"{ value2(values: ["a", "cdef"])}"#)
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: r#"Failed to parse "String": the string length is 4, must be less than or equal to 3"#.to_string(),
            source: None,
            locations: vec![Pos { column: 18, line: 1}],
            path: vec![PathSegment::Field("value2".to_string())],
            extensions: None
        }]
    );

    assert_eq!(
        schema
            .execute(r#"{ value2(values: ["a", "b", "cdef"])}"#)
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: r#"Failed to parse "[String!]": the value length is 3, must be less than or equal to 2"#.to_string(),
            source: None,
            locations: vec![Pos { column: 18, line: 1}],
            path: vec![PathSegment::Field("value2".to_string())],
            extensions: None
        }]
    );

    assert_eq!(
        schema
            .execute(r#"{ value2(values: null)}"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "value2": ""
        })
    );
}

#[tokio::test]
pub async fn test_issue_1164() {
    struct PasswordValidator;

    impl CustomValidator<String> for PasswordValidator {
        /// Check if `value` only contains allowed chars
        fn check(&self, value: &String) -> Result<(), InputValueError<String>> {
            let allowed_chars = ['1', '2', '3', '4', '5', '6'];

            if value
                .chars()
                .all(|c| allowed_chars.contains(&c.to_ascii_lowercase()))
            {
                Ok(())
            } else {
                Err(InputValueError::custom(format!(
                    "illegal char in password: `{}`",
                    value
                )))
            }
        }
    }

    struct Query;

    #[Object]
    impl Query {
        async fn a(
            &self,
            #[graphql(validator(min_length = 6, max_length = 16, custom = "PasswordValidator"))]
            value: String,
        ) -> String {
            value
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    assert_eq!(
        schema
            .execute(r#"{ a(value: "123456")}"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "a": "123456"
        })
    );

    assert_eq!(
        schema
            .execute(r#"{ a(value: "123") }"#)
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: r#"Failed to parse "String": the string length is 3, must be greater than or equal to 6"#.to_string(),
            source: None,
            locations: vec![Pos { column: 12, line: 1}],
            path: vec![PathSegment::Field("a".to_string())],
            extensions: None
        }]
    );

    assert_eq!(
        schema
            .execute(r#"{ a(value: "123123123123123123") }"#)
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: r#"Failed to parse "String": the string length is 18, must be less than or equal to 16"#.to_string(),
            source: None,
            locations: vec![Pos { column: 12, line: 1}],
            path: vec![PathSegment::Field("a".to_string())],
            extensions: None
        }]
    );

    assert_eq!(
        schema
            .execute(r#"{ a(value: "abcdef") }"#)
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: r#"Failed to parse "String": illegal char in password: `abcdef`"#.to_string(),
            source: None,
            locations: vec![Pos {
                column: 12,
                line: 1
            }],
            path: vec![PathSegment::Field("a".to_string())],
            extensions: None
        }]
    );
}
